//! Tests for saga macros

use allframe_core::cqrs::{Saga, StepExecutionResult, StepOutput};
use allframe_macros::{saga, saga_data, StepOutput as StepOutputDerive};
use serde::{Deserialize, Serialize};

// Test data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestSagaData {
    user_id: String,
    amount: f64,
}

// Test step output
#[derive(Serialize, Deserialize, StepOutputDerive)]
struct PaymentResult {
    transaction_id: String,
    status: String,
}

// Test saga container
#[allframe_macros::saga(name = "TestPaymentSaga", data_field = "data")]
struct TestPaymentSaga {
    data: TestSagaData,
    // Add some mock dependencies for the workflow
    index_repository: std::sync::Arc<dyn std::any::Any + Send + Sync>,
    trade_service: std::sync::Arc<dyn std::any::Any + Send + Sync>,
}

// Test workflow
#[allframe_macros::saga_workflow(TestPaymentSaga)]
enum TestPaymentWorkflow {
    ValidatePayment,
    ProcessPayment,
    SendConfirmation,
}

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

// #[test]
// fn test_saga_macro_compilation() {
//     // Just test that the struct and trait impls compile
//     // The constructor signature might need adjustment
//     let data = TestSagaData {
//         user_id: "user123".to_string(),
//         amount: 100.0,
//     };
//
//     // For now, just test that we can create the data
//     assert_eq!(data.user_id, "user123");
//     assert_eq!(data.amount, 100.0);
// }
