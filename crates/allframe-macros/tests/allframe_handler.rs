//! Integration tests for #[allframe_handler] attribute macro

use allframe_macros::allframe_handler;

// ─── Basic handlers ──────────────────────────────────────────────────────

#[allframe_handler]
async fn get_user() -> String {
    r#"{"name":"Alice"}"#.to_string()
}

#[allframe_handler]
async fn echo(input: String) -> String {
    input
}

#[allframe_handler]
pub async fn public_handler() -> String {
    "public".to_string()
}

#[tokio::test]
async fn test_basic_handler() {
    let result = get_user().await;
    assert_eq!(result, r#"{"name":"Alice"}"#);
}

#[tokio::test]
async fn test_handler_with_args() {
    let result = echo("hello".to_string()).await;
    assert_eq!(result, "hello");
}

#[tokio::test]
async fn test_public_handler() {
    let result = public_handler().await;
    assert_eq!(result, "public");
}

// ─── Streaming handlers ─────────────────────────────────────────────────

use allframe_core::router::StreamSender;

#[allframe_handler(streaming)]
async fn stream_data(tx: StreamSender) -> String {
    tx.send("chunk1".to_string()).await.ok();
    tx.send("chunk2".to_string()).await.ok();
    "done".to_string()
}

#[allframe_handler(streaming)]
async fn stream_with_args(prefix: String, tx: StreamSender) -> String {
    tx.send(format!("{prefix}-item")).await.ok();
    "complete".to_string()
}

// Verify streaming handlers compile and are callable
// (Full streaming tests require a Router, covered in allframe-tauri tests)

// ─── Combined with tauri_compat ──────────────────────────────────────────

use allframe_macros::tauri_compat;

#[allframe_handler]
#[tauri_compat]
async fn greet(name: String, age: u32) -> String {
    format!("Hello {}, age {}", name, age)
}

#[tokio::test]
async fn test_combined_with_tauri_compat() {
    let args = GreetArgs {
        name: "Bob".to_string(),
        age: 25,
    };
    let result = greet(args).await;
    assert_eq!(result, "Hello Bob, age 25");
}
