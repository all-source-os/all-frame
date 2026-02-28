//! Saga pattern implementation with macro support
//!
//! This module provides the trait definitions and types used by the saga macros
//! in allframe-macros. It provides a higher-level, macro-driven approach
//! compared to the lower-level saga_orchestrator.

#![cfg(feature = "cqrs")]

use std::{collections::HashMap, sync::Arc};

use serde_json;

/// A saga that coordinates multiple steps with automatic compensation
#[async_trait::async_trait]
pub trait Saga: Send + Sync {
    /// Get the saga type name for identification
    fn saga_type(&self) -> &'static str;

    /// Get all steps in execution order
    fn steps(&self) -> Vec<Arc<dyn SagaStep>>;

    /// Get the initial saga data as JSON
    fn initial_data(&self) -> serde_json::Value;

    /// Get the user ID associated with this saga
    fn user_id(&self) -> &str;
}

/// A single step in a saga
#[async_trait::async_trait]
pub trait SagaStep: Send + Sync {
    /// Execute the step
    async fn execute(&self, ctx: &SagaContext) -> StepExecutionResult;

    /// Compensate for this step (rollback)
    async fn compensate(&self, ctx: &SagaContext) -> CompensationResult;

    /// Get the step name for logging/debugging
    fn name(&self) -> &str;

    /// Get the timeout for this step in seconds
    fn timeout_seconds(&self) -> u64 {
        30 // Default 30 seconds
    }

    /// Whether this step requires compensation on failure
    fn requires_compensation(&self) -> bool {
        true // Default to requiring compensation
    }
}

/// Context passed to saga steps during execution
#[derive(Debug, Clone)]
pub struct SagaContext {
    /// Unique saga instance ID
    pub saga_id: String,
    /// Step outputs from previously executed steps
    pub step_outputs: HashMap<String, serde_json::Value>,
    /// Saga metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl SagaContext {
    /// Create a new saga context
    pub fn new(saga_id: String) -> Self {
        Self {
            saga_id,
            step_outputs: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Store output from a completed step
    pub fn set_step_output(&mut self, step_name: &str, output: serde_json::Value) {
        self.step_outputs.insert(step_name.to_string(), output);
    }

    /// Get output from a previously executed step
    pub fn get_step_output(&self, step_name: &str) -> Option<&serde_json::Value> {
        self.step_outputs.get(step_name)
    }

    /// Store arbitrary metadata
    pub fn set_metadata(&mut self, key: &str, value: serde_json::Value) {
        self.metadata.insert(key.to_string(), value);
    }

    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }
}

/// Result of a step execution
#[derive(Debug, Clone)]
pub enum StepExecutionResult {
    /// Step completed successfully
    Success {
        /// Optional output from the step
        output: Option<serde_json::Value>,
    },
    /// Step failed
    Failure {
        /// Error message describing the failure
        error: String,
    },
}

impl StepExecutionResult {
    /// Create a success result with no output
    pub fn success() -> Self {
        Self::Success { output: None }
    }

    /// Create a success result with output
    pub fn success_with_output(output: serde_json::Value) -> Self {
        Self::Success {
            output: Some(output),
        }
    }

    /// Create a failure result
    pub fn failure(error: String) -> Self {
        Self::Failure { error }
    }

    /// Check if the result is successful
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    /// Get the output if successful
    pub fn output(&self) -> Option<&serde_json::Value> {
        match self {
            Self::Success { output } => output.as_ref(),
            Self::Failure { .. } => None,
        }
    }

    /// Get the error if failed
    pub fn error(&self) -> Option<&str> {
        match self {
            Self::Success { .. } => None,
            Self::Failure { error } => Some(error),
        }
    }
}

/// Result of a compensation operation
#[derive(Debug, Clone)]
pub enum CompensationResult {
    /// Compensation completed successfully
    Success,
    /// Compensation failed
    Failure {
        /// Error message describing the failure
        error: String,
    },
    /// No compensation needed
    NotNeeded,
}

impl CompensationResult {
    /// Create a success result
    pub fn success() -> Self {
        Self::Success
    }

    /// Create a failure result
    pub fn failure(error: String) -> Self {
        Self::Failure { error }
    }

    /// Create a not needed result
    pub fn not_needed() -> Self {
        Self::NotNeeded
    }

    /// Check if compensation was successful
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }

    /// Check if compensation is not needed
    pub fn is_not_needed(&self) -> bool {
        matches!(self, Self::NotNeeded)
    }
}

/// Trait for types that can be used as step outputs
pub trait StepOutput: serde::de::DeserializeOwned + serde::Serialize {
    /// Extract this type from saga context
    fn from_context(ctx: &SagaContext, step_name: &str) -> Result<Self, SagaError>;
}

/// Errors that can occur during saga operations
#[derive(Debug, Clone)]
pub enum SagaError {
    /// Step output not found
    StepOutputNotFound {
        /// Name of the step whose output was not found
        step_name: String,
    },
    /// Failed to parse step output
    StepOutputParse {
        /// Name of the step whose output failed to parse
        step_name: String,
        /// Parse error message
        error: String,
    },
    /// Step execution failed
    StepExecutionFailed {
        /// Name of the step that failed
        step_name: String,
        /// Execution error message
        error: String,
    },
    /// Compensation failed
    CompensationFailed {
        /// Name of the step whose compensation failed
        step_name: String,
        /// Compensation error message
        error: String,
    },
    /// Saga not found
    SagaNotFound {
        /// ID of the saga that was not found
        saga_id: String,
    },
    /// Invalid saga state
    InvalidState {
        /// ID of the saga with invalid state
        saga_id: String,
        /// Description of the invalid state
        message: String,
    },
}

impl std::fmt::Display for SagaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SagaError::StepOutputNotFound { step_name } => {
                write!(f, "Step output not found for step: {}", step_name)
            }
            SagaError::StepOutputParse { step_name, error } => {
                write!(
                    f,
                    "Failed to parse output for step {}: {}",
                    step_name, error
                )
            }
            SagaError::StepExecutionFailed { step_name, error } => {
                write!(f, "Step {} execution failed: {}", step_name, error)
            }
            SagaError::CompensationFailed { step_name, error } => {
                write!(f, "Compensation failed for step {}: {}", step_name, error)
            }
            SagaError::SagaNotFound { saga_id } => {
                write!(f, "Saga not found: {}", saga_id)
            }
            SagaError::InvalidState { saga_id, message } => {
                write!(f, "Invalid saga state for {}: {}", saga_id, message)
            }
        }
    }
}

impl std::error::Error for SagaError {}

impl<E> From<SagaError> for Result<E, SagaError> {
    fn from(error: SagaError) -> Self {
        Err(error)
    }
}

/// Saga orchestrator for executing macro-generated sagas
pub struct MacroSagaOrchestrator {
    // Future: Add persistence, monitoring, etc.
}

impl Default for MacroSagaOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl MacroSagaOrchestrator {
    /// Create a new orchestrator
    pub fn new() -> Self {
        Self {}
    }

    /// Execute a saga (placeholder implementation)
    pub async fn execute(&self, _saga: Arc<dyn Saga>) -> Result<(), SagaError> {
        // TODO: Implement saga execution logic
        // This is a placeholder until the full orchestrator is implemented
        Ok(())
    }
}

// ============================================================================
// UC-036.7: Saga Compensation Primitives
// ============================================================================

/// A snapshot of a file's contents, used for rollback compensation.
#[derive(Debug, Clone)]
pub struct FileSnapshot {
    /// Path to the file
    pub path: std::path::PathBuf,
    /// Original content
    pub content: Vec<u8>,
}

impl FileSnapshot {
    /// Capture the current content of a file.
    pub async fn capture(path: &std::path::Path) -> Result<Self, String> {
        let path_buf = path.to_path_buf();
        let content = tokio::fs::read(&path_buf)
            .await
            .map_err(|e| format!("FileSnapshot capture: {}", e))?;
        Ok(Self {
            path: path_buf,
            content,
        })
    }

    /// Restore the file to its captured content.
    pub async fn restore(&self) -> Result<(), String> {
        tokio::fs::write(&self.path, &self.content)
            .await
            .map_err(|e| format!("FileSnapshot restore: {}", e))
    }
}

/// Strategy for compensating saga steps on failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompensationStrategy {
    /// Roll back using local file snapshots / savepoints.
    LocalRollback,
}

/// A saga step that writes content to a file, with automatic snapshot for compensation.
pub struct WriteFileStep {
    /// Path to write to.
    pub path: std::path::PathBuf,
    /// Content to write.
    pub content: String,
    /// Internal snapshot taken before execution.
    snapshot: tokio::sync::Mutex<Option<FileSnapshot>>,
}

impl WriteFileStep {
    /// Create a new WriteFileStep (convenience for tests that set path/content directly).
    pub fn new(path: std::path::PathBuf, content: String) -> Self {
        Self {
            path,
            content,
            snapshot: tokio::sync::Mutex::new(None),
        }
    }
}

#[async_trait::async_trait]
impl<E: super::Event> super::saga_orchestrator::SagaStep<E> for WriteFileStep {
    async fn execute(&self) -> Result<Vec<E>, String> {
        // Capture snapshot before writing
        if self.path.exists() {
            let snap = FileSnapshot::capture(&self.path).await?;
            *self.snapshot.lock().await = Some(snap);
        }
        tokio::fs::write(&self.path, &self.content)
            .await
            .map_err(|e| format!("WriteFileStep: {}", e))?;
        Ok(vec![])
    }

    async fn compensate(&self) -> Result<Vec<E>, String> {
        if let Some(snap) = self.snapshot.lock().await.as_ref() {
            snap.restore().await?;
        }
        Ok(vec![])
    }

    fn name(&self) -> &str {
        "WriteFileStep"
    }
}

/// SQLite savepoint for transactional rollback within a saga step.
///
/// Holds a reference to the connection and the savepoint name.
/// The caller must ensure the `SqliteSavepoint` does not outlive the connection.
#[cfg(feature = "cqrs-sqlite")]
pub struct SqliteSavepoint<'conn> {
    conn: &'conn rusqlite::Connection,
    name: String,
}

#[cfg(feature = "cqrs-sqlite")]
impl<'conn> SqliteSavepoint<'conn> {
    /// Create a savepoint on the given connection.
    pub fn create(conn: &'conn rusqlite::Connection, name: &str) -> Result<Self, String> {
        conn.execute_batch(&format!("SAVEPOINT {}", name))
            .map_err(|e| format!("Savepoint create: {}", e))?;
        Ok(Self {
            conn,
            name: name.to_string(),
        })
    }

    /// Rollback to this savepoint.
    pub fn rollback(&self) -> Result<(), String> {
        self.conn
            .execute_batch(&format!("ROLLBACK TO SAVEPOINT {}", self.name))
            .map_err(|e| format!("Savepoint rollback: {}", e))
    }
}
