//! Integration tests for TypeScript codegen output for streaming handlers

use allframe_core::router::{
    generate_ts_client, HandlerMeta, Router, StreamSender, TsField, TsType,
};
use std::collections::HashMap;

#[test]
fn test_streaming_codegen_generates_infrastructure() {
    let mut metas = HashMap::new();
    metas.insert(
        "stream_chat".to_string(),
        HandlerMeta::streaming(
            vec![TsField::new("prompt", TsType::String)],
            TsType::String,
            TsType::Boolean,
        ),
    );

    let ts = generate_ts_client(&metas);

    // Streaming infrastructure
    assert!(ts.contains("import { listen"));
    assert!(ts.contains("export interface StreamObserver"));
    assert!(ts.contains("export interface StreamSubscription"));
    assert!(ts.contains("async function callStreamHandler"));
    assert!(ts.contains("allframe_stream"));

    // RxJS adapter
    assert!(ts.contains("export async function toObservable"));
    assert!(ts.contains("import(\"rxjs\")"));
}

#[test]
fn test_streaming_codegen_function_signature() {
    let mut metas = HashMap::new();
    metas.insert(
        "stream_chat".to_string(),
        HandlerMeta::streaming(
            vec![TsField::new("prompt", TsType::String)],
            TsType::String,
            TsType::Boolean,
        ),
    );

    let ts = generate_ts_client(&metas);

    assert!(ts.contains("export interface StreamChatArgs {"));
    assert!(ts.contains("  prompt: string;"));
    assert!(ts.contains("export async function streamChat(args: StreamChatArgs, observer: StreamObserver<string, boolean>): Promise<StreamSubscription>"));
    assert!(ts.contains("callStreamHandler<string, boolean>(\"stream_chat\", args, observer)"));
}

#[test]
fn test_streaming_codegen_no_args() {
    let mut metas = HashMap::new();
    metas.insert(
        "stream_updates".to_string(),
        HandlerMeta::streaming(vec![], TsType::Number, TsType::Void),
    );

    let ts = generate_ts_client(&metas);

    assert!(ts.contains("export async function streamUpdates(observer: StreamObserver<number, void>): Promise<StreamSubscription>"));
    assert!(ts.contains("callStreamHandler<number, void>(\"stream_updates\", {}, observer)"));
}

#[test]
fn test_mixed_codegen_regular_and_streaming() {
    let mut metas = HashMap::new();
    metas.insert(
        "get_user".to_string(),
        HandlerMeta::new(
            vec![TsField::new("id", TsType::Number)],
            TsType::String,
        ),
    );
    metas.insert(
        "stream_events".to_string(),
        HandlerMeta::streaming(vec![], TsType::String, TsType::Void),
    );

    let ts = generate_ts_client(&metas);

    // Regular handler
    assert!(ts.contains("callHandler<string>(\"get_user\""));
    // Streaming handler
    assert!(ts.contains("callStreamHandler<string, void>(\"stream_events\""));
    // Both helpers present
    assert!(ts.contains("async function callHandler"));
    assert!(ts.contains("async function callStreamHandler"));
}

#[test]
fn test_no_streaming_infra_without_streaming_handlers() {
    let mut metas = HashMap::new();
    metas.insert(
        "get_user".to_string(),
        HandlerMeta::new(vec![], TsType::String),
    );

    let ts = generate_ts_client(&metas);
    assert!(!ts.contains("StreamObserver"));
    assert!(!ts.contains("callStreamHandler"));
    assert!(!ts.contains("toObservable"));
}

#[test]
fn test_codegen_via_router_describe_streaming() {
    let mut router = Router::new();
    router.register_streaming("my_stream", |_tx: StreamSender| async move {
        "done".to_string()
    });
    router.describe_streaming_handler(
        "my_stream",
        vec![TsField::new("query", TsType::String)],
        TsType::Object(vec![TsField::new("token", TsType::String)]),
        TsType::Boolean,
    );

    let ts = router.generate_ts_client();
    assert!(ts.contains("export interface MyStreamArgs {"));
    assert!(ts.contains("export async function myStream(args: MyStreamArgs"));
    assert!(ts.contains("StreamObserver"));
}
