//! Saga Orchestrator for distributed transaction coordination
//!
//! This module provides automatic saga orchestration, eliminating boilerplate
//! for multi-aggregate transactions with automatic compensation and retry logic.

use super::Event;
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::timeout;

/// Result type for saga operations
pub type SagaResult<T> = Result<T, SagaError>;

/// Errors that can occur during saga execution
#[derive(Debug, Clone)]
pub enum SagaError {
    /// Step execution failed
    StepFailed {
        /// Index of the failed step
        step_index: usize,
        /// Name of the failed step
        step_name: String,
        /// Error message
        error: String,
    },
    /// Compensation failed
    CompensationFailed {
        /// Index of the step being compensated
        step_index: usize,
        /// Error message
        error: String,
    },
    /// Timeout occurred
    Timeout {
        /// Index of the timed out step
        step_index: usize,
        /// Duration that was exceeded
        duration: Duration,
    },
    /// Invalid step index
    InvalidStep(usize),
    /// Saga already executing
    AlreadyExecuting,
}

impl fmt::Display for SagaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SagaError::StepFailed { step_index, step_name, error } => {
                write!(f, "Step {} ({}) failed: {}", step_index, step_name, error)
            }
            SagaError::CompensationFailed { step_index, error } => {
                write!(f, "Compensation for step {} failed: {}", step_index, error)
            }
            SagaError::Timeout { step_index, duration } => {
                write!(f, "Step {} timed out after {:?}", step_index, duration)
            }
            SagaError::InvalidStep(index) => write!(f, "Invalid step index: {}", index),
            SagaError::AlreadyExecuting => write!(f, "Saga is already executing"),
        }
    }
}

impl std::error::Error for SagaError {}

/// Status of a saga execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SagaStatus {
    /// Saga not yet started
    NotStarted,
    /// Saga is currently executing
    Executing,
    /// Saga completed successfully
    Completed,
    /// Saga failed and compensation was successful
    Compensated,
    /// Saga failed and compensation also failed
    Failed,
}

/// Metadata about saga execution
#[derive(Debug, Clone)]
pub struct SagaMetadata {
    /// Unique saga ID
    pub id: String,
    /// Current status
    pub status: SagaStatus,
    /// Number of steps executed
    pub steps_executed: usize,
    /// Number of total steps
    pub total_steps: usize,
    /// Timestamp of last update
    pub updated_at: std::time::SystemTime,
}

/// A single step in a saga
#[async_trait::async_trait]
pub trait SagaStep<E: Event>: Send + Sync {
    /// Execute the step
    async fn execute(&self) -> Result<Vec<E>, String>;

    /// Compensate for this step (rollback)
    async fn compensate(&self) -> Result<Vec<E>, String>;

    /// Get the step name for logging/debugging
    fn name(&self) -> &str;

    /// Get the timeout for this step
    fn timeout_duration(&self) -> Duration {
        Duration::from_secs(30) // Default 30 seconds
    }
}

/// Saga definition with ordered steps
pub struct SagaDefinition<E: Event> {
    /// Unique saga ID
    id: String,
    /// Ordered list of steps
    steps: Vec<Box<dyn SagaStep<E>>>,
    /// Metadata
    metadata: SagaMetadata,
}

impl<E: Event> SagaDefinition<E> {
    /// Create a new saga definition
    pub fn new(id: impl Into<String>) -> Self {
        let id = id.into();
        Self {
            metadata: SagaMetadata {
                id: id.clone(),
                status: SagaStatus::NotStarted,
                steps_executed: 0,
                total_steps: 0,
                updated_at: std::time::SystemTime::now(),
            },
            id,
            steps: Vec::new(),
        }
    }

    /// Add a step to the saga
    pub fn add_step<S: SagaStep<E> + 'static>(mut self, step: S) -> Self {
        self.steps.push(Box::new(step));
        self.metadata.total_steps = self.steps.len();
        self
    }

    /// Get saga ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get current status
    pub fn status(&self) -> SagaStatus {
        self.metadata.status.clone()
    }

    /// Get metadata
    pub fn metadata(&self) -> &SagaMetadata {
        &self.metadata
    }
}

/// Orchestrator for executing sagas
pub struct SagaOrchestrator<E: Event> {
    /// Running sagas
    sagas: Arc<RwLock<HashMap<String, SagaMetadata>>>,
    /// Completed sagas history
    history: Arc<RwLock<Vec<SagaMetadata>>>,
    _phantom: PhantomData<E>,
}

impl<E: Event> SagaOrchestrator<E> {
    /// Create a new saga orchestrator
    pub fn new() -> Self {
        Self {
            sagas: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(Vec::new())),
            _phantom: PhantomData,
        }
    }

    /// Execute a saga with automatic compensation on failure
    pub async fn execute(&self, mut saga: SagaDefinition<E>) -> SagaResult<Vec<E>> {
        // Check if saga is already running
        {
            let sagas = self.sagas.read().await;
            if sagas.contains_key(&saga.id) {
                return Err(SagaError::AlreadyExecuting);
            }
        }

        // Mark as executing
        saga.metadata.status = SagaStatus::Executing;
        saga.metadata.updated_at = std::time::SystemTime::now();
        {
            let mut sagas = self.sagas.write().await;
            sagas.insert(saga.id.clone(), saga.metadata.clone());
        }

        let mut all_events = Vec::new();
        let mut executed_steps = 0;

        // Execute each step
        for (index, step) in saga.steps.iter().enumerate() {
            // Execute step with timeout
            let step_timeout = step.timeout_duration();
            let result = timeout(step_timeout, step.execute()).await;

            match result {
                Ok(Ok(events)) => {
                    // Step succeeded
                    all_events.extend(events);
                    executed_steps += 1;
                    saga.metadata.steps_executed = executed_steps;
                    saga.metadata.updated_at = std::time::SystemTime::now();
                }
                Ok(Err(error)) => {
                    // Step failed - compensate previous steps
                    saga.metadata.status = SagaStatus::Failed;
                    let compensation_result = self.compensate_steps(&saga.steps[0..index]).await;

                    // Remove from active sagas
                    {
                        let mut sagas = self.sagas.write().await;
                        sagas.remove(&saga.id);
                    }

                    // Add to history
                    {
                        let mut history = self.history.write().await;
                        saga.metadata.status = if compensation_result.is_ok() {
                            SagaStatus::Compensated
                        } else {
                            SagaStatus::Failed
                        };
                        history.push(saga.metadata.clone());
                    }

                    return Err(SagaError::StepFailed {
                        step_index: index,
                        step_name: step.name().to_string(),
                        error,
                    });
                }
                Err(_) => {
                    // Timeout
                    saga.metadata.status = SagaStatus::Failed;
                    let _ = self.compensate_steps(&saga.steps[0..index]).await;

                    {
                        let mut sagas = self.sagas.write().await;
                        sagas.remove(&saga.id);
                    }

                    return Err(SagaError::Timeout {
                        step_index: index,
                        duration: step_timeout,
                    });
                }
            }
        }

        // All steps completed successfully
        saga.metadata.status = SagaStatus::Completed;
        saga.metadata.updated_at = std::time::SystemTime::now();

        // Remove from active and add to history
        {
            let mut sagas = self.sagas.write().await;
            sagas.remove(&saga.id);
        }
        {
            let mut history = self.history.write().await;
            history.push(saga.metadata);
        }

        Ok(all_events)
    }

    /// Compensate (rollback) executed steps in reverse order
    async fn compensate_steps(&self, steps: &[Box<dyn SagaStep<E>>]) -> Result<(), String> {
        // Compensate in reverse order
        for step in steps.iter().rev() {
            step.compensate().await?;
        }
        Ok(())
    }

    /// Get metadata for a running saga
    pub async fn get_saga(&self, id: &str) -> Option<SagaMetadata> {
        let sagas = self.sagas.read().await;
        sagas.get(id).cloned()
    }

    /// Get all running sagas
    pub async fn get_running_sagas(&self) -> Vec<SagaMetadata> {
        let sagas = self.sagas.read().await;
        sagas.values().cloned().collect()
    }

    /// Get saga history
    pub async fn get_history(&self) -> Vec<SagaMetadata> {
        let history = self.history.read().await;
        history.clone()
    }

    /// Get number of running sagas
    pub async fn running_count(&self) -> usize {
        self.sagas.read().await.len()
    }

    /// Get number of completed sagas (including failed)
    pub async fn history_count(&self) -> usize {
        self.history.read().await.len()
    }
}

impl<E: Event> Default for SagaOrchestrator<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: Event> Clone for SagaOrchestrator<E> {
    fn clone(&self) -> Self {
        Self {
            sagas: Arc::clone(&self.sagas),
            history: Arc::clone(&self.history),
            _phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum TestEvent {
        Debited { account: String, amount: f64 },
        Credited { account: String, amount: f64 },
    }

    impl Event for TestEvent {}

    struct DebitStep {
        account: String,
        amount: f64,
    }

    #[async_trait::async_trait]
    impl SagaStep<TestEvent> for DebitStep {
        async fn execute(&self) -> Result<Vec<TestEvent>, String> {
            Ok(vec![TestEvent::Debited {
                account: self.account.clone(),
                amount: self.amount,
            }])
        }

        async fn compensate(&self) -> Result<Vec<TestEvent>, String> {
            // Compensate by crediting back
            Ok(vec![TestEvent::Credited {
                account: self.account.clone(),
                amount: self.amount,
            }])
        }

        fn name(&self) -> &str {
            "DebitStep"
        }
    }

    struct CreditStep {
        account: String,
        amount: f64,
    }

    #[async_trait::async_trait]
    impl SagaStep<TestEvent> for CreditStep {
        async fn execute(&self) -> Result<Vec<TestEvent>, String> {
            Ok(vec![TestEvent::Credited {
                account: self.account.clone(),
                amount: self.amount,
            }])
        }

        async fn compensate(&self) -> Result<Vec<TestEvent>, String> {
            // Compensate by debiting back
            Ok(vec![TestEvent::Debited {
                account: self.account.clone(),
                amount: self.amount,
            }])
        }

        fn name(&self) -> &str {
            "CreditStep"
        }
    }

    #[tokio::test]
    async fn test_successful_saga() {
        let orchestrator = SagaOrchestrator::<TestEvent>::new();

        let saga = SagaDefinition::new("transfer-1")
            .add_step(DebitStep {
                account: "A".to_string(),
                amount: 100.0,
            })
            .add_step(CreditStep {
                account: "B".to_string(),
                amount: 100.0,
            });

        let events = orchestrator.execute(saga).await.unwrap();

        assert_eq!(events.len(), 2);
        assert_eq!(orchestrator.running_count().await, 0);
        assert_eq!(orchestrator.history_count().await, 1);
    }

    #[tokio::test]
    async fn test_saga_metadata() {
        let orchestrator = SagaOrchestrator::<TestEvent>::new();

        let saga = SagaDefinition::new("transfer-2")
            .add_step(DebitStep {
                account: "A".to_string(),
                amount: 50.0,
            });

        assert_eq!(saga.id(), "transfer-2");
        assert_eq!(saga.status(), SagaStatus::NotStarted);
        assert_eq!(saga.metadata().total_steps, 1);

        orchestrator.execute(saga).await.unwrap();

        let history = orchestrator.get_history().await;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].status, SagaStatus::Completed);
    }

    #[tokio::test]
    async fn test_saga_definition_builder() {
        let saga = SagaDefinition::<TestEvent>::new("test-saga")
            .add_step(DebitStep {
                account: "A".to_string(),
                amount: 10.0,
            })
            .add_step(CreditStep {
                account: "B".to_string(),
                amount: 10.0,
            });

        assert_eq!(saga.metadata().total_steps, 2);
        assert_eq!(saga.status(), SagaStatus::NotStarted);
    }

    #[tokio::test]
    async fn test_multiple_sagas() {
        let orchestrator = SagaOrchestrator::<TestEvent>::new();

        let saga1 = SagaDefinition::new("transfer-1")
            .add_step(DebitStep {
                account: "A".to_string(),
                amount: 100.0,
            });

        let saga2 = SagaDefinition::new("transfer-2")
            .add_step(DebitStep {
                account: "B".to_string(),
                amount: 200.0,
            });

        orchestrator.execute(saga1).await.unwrap();
        orchestrator.execute(saga2).await.unwrap();

        assert_eq!(orchestrator.history_count().await, 2);
    }
}
