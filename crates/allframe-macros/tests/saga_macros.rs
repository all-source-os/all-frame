//! Tests for saga macros
#![allow(dead_code)]

use allframe_core::cqrs::{
    CompensationResult, MacroSagaStep, Saga, SagaContext, StepExecutionResult,
};
use allframe_macros::StepOutput as StepOutputDerive;
use serde::{Deserialize, Serialize};

// ─── Test data structures ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestSagaData {
    user_id: String,
    amount: f64,
}

// ─── Step output ───────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, StepOutputDerive)]
struct PaymentResult {
    transaction_id: String,
    status: String,
}

// ─── Step structs required by #[saga_workflow] ─────────────────────────────

#[derive(Default)]
struct ValidatePaymentStep;

#[async_trait::async_trait]
impl MacroSagaStep for ValidatePaymentStep {
    async fn execute(&self, _ctx: &SagaContext) -> StepExecutionResult {
        StepExecutionResult::Success { output: None }
    }
    async fn compensate(&self, _ctx: &SagaContext) -> CompensationResult {
        CompensationResult::Success
    }
    fn name(&self) -> &str {
        "ValidatePayment"
    }
}

#[derive(Default)]
struct ProcessPaymentStep;

#[async_trait::async_trait]
impl MacroSagaStep for ProcessPaymentStep {
    async fn execute(&self, _ctx: &SagaContext) -> StepExecutionResult {
        StepExecutionResult::Success { output: None }
    }
    async fn compensate(&self, _ctx: &SagaContext) -> CompensationResult {
        CompensationResult::Success
    }
    fn name(&self) -> &str {
        "ProcessPayment"
    }
}

#[derive(Default)]
struct SendConfirmationStep;

#[async_trait::async_trait]
impl MacroSagaStep for SendConfirmationStep {
    async fn execute(&self, _ctx: &SagaContext) -> StepExecutionResult {
        StepExecutionResult::Success { output: None }
    }
    async fn compensate(&self, _ctx: &SagaContext) -> CompensationResult {
        CompensationResult::Success
    }
    fn name(&self) -> &str {
        "SendConfirmation"
    }
}

// ─── Saga container (standalone, no workflow) ─��────────────────────────────

#[allframe_macros::saga(name = "TestPaymentSaga", data_field = "data")]
struct TestPaymentSaga {
    data: TestSagaData,
    #[inject]
    index_repository: std::sync::Arc<dyn std::any::Any + Send + Sync>,
    #[inject]
    trade_service: std::sync::Arc<dyn std::any::Any + Send + Sync>,
}

// ─── Saga with workflow ───────────────────────────────────────────���────────

#[allframe_macros::saga(name = "WorkflowPaymentSaga", data_field = "data")]
struct WorkflowPaymentSaga {
    data: TestSagaData,
}

#[allframe_macros::saga_workflow(WorkflowPaymentSaga)]
enum WorkflowPaymentWorkflow {
    ValidatePayment,
    ProcessPayment,
    SendConfirmation,
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[test]
fn test_step_output_derive() {
    let output = PaymentResult {
        transaction_id: "tx_123".to_string(),
        status: "completed".to_string(),
    };

    let json = serde_json::to_value(&output).unwrap();
    assert_eq!(json["transaction_id"], "tx_123");
    assert_eq!(json["status"], "completed");

    // Test From conversion
    let result: StepExecutionResult = output.into();
    assert!(matches!(
        result,
        StepExecutionResult::Success { output: Some(_) }
    ));
}

#[test]
fn test_saga_macro_creates_constructor_and_trait_impl() {
    let data = TestSagaData {
        user_id: "user123".to_string(),
        amount: 100.0,
    };
    let repo: std::sync::Arc<dyn std::any::Any + Send + Sync> = std::sync::Arc::new(());
    let svc: std::sync::Arc<dyn std::any::Any + Send + Sync> = std::sync::Arc::new(());

    let saga = TestPaymentSaga::new(data, repo, svc);
    assert_eq!(saga.saga_type(), "TestPaymentSaga");
    assert_eq!(saga.user_id(), "user123");
    // Standalone saga (no workflow) returns empty steps
    assert!(saga.steps().is_empty());
}

#[test]
fn test_saga_workflow_generates_steps() {
    let data = TestSagaData {
        user_id: "user456".to_string(),
        amount: 50.0,
    };
    let saga = WorkflowPaymentSaga::new(data);
    assert_eq!(saga.saga_type(), "WorkflowPaymentSaga");

    let steps = saga.workflow_steps().expect("workflow should provide steps");
    assert_eq!(steps.len(), 3);
    assert_eq!(steps[0].name(), "ValidatePayment");
    assert_eq!(steps[1].name(), "ProcessPayment");
    assert_eq!(steps[2].name(), "SendConfirmation");
}
