//! Integration tests for TauriServer streaming, cancellation, concurrent streams

use allframe_core::router::{Router, StreamSender};
use allframe_tauri::{HandlerKind, TauriServer, TauriServerError};

#[tokio::test]
async fn test_tauri_server_streaming_handler() {
    let mut router = Router::new();
    router.register_streaming("stream", |tx: StreamSender| async move {
        tx.send("hello".to_string()).await.ok();
        tx.send("world".to_string()).await.ok();
        "final".to_string()
    });

    let server = TauriServer::new(router);

    let (mut rx, handle) = server.call_streaming_handler("stream", "{}").unwrap();

    let result = handle.await.unwrap().unwrap();
    assert_eq!(result.result, "final");

    assert_eq!(rx.recv().await, Some("hello".to_string()));
    assert_eq!(rx.recv().await, Some("world".to_string()));
}

#[tokio::test]
async fn test_tauri_server_streaming_not_found() {
    let router = Router::new();
    let server = TauriServer::new(router);

    match server.call_streaming_handler("missing", "{}") {
        Err(TauriServerError::HandlerNotFound(name)) => assert_eq!(name, "missing"),
        other => panic!("Expected HandlerNotFound, got: {other:?}"),
    }
}

#[tokio::test]
async fn test_tauri_server_streaming_wrong_kind() {
    let mut router = Router::new();
    router.register("regular", || async { "ok".to_string() });

    let server = TauriServer::new(router);

    match server.call_streaming_handler("regular", "{}") {
        Err(TauriServerError::NotStreamingHandler(name)) => assert_eq!(name, "regular"),
        other => panic!("Expected NotStreamingHandler, got: {other:?}"),
    }
}

#[tokio::test]
async fn test_tauri_server_handler_kind_reported() {
    let mut router = Router::new();
    router.register("regular", || async { "ok".to_string() });
    router.register_streaming("stream", |_tx: StreamSender| async move {
        "ok".to_string()
    });

    let server = TauriServer::new(router);
    let handlers = server.list_handlers();

    let regular = handlers.iter().find(|h| h.name == "regular").unwrap();
    assert_eq!(regular.kind, HandlerKind::RequestResponse);

    let stream = handlers.iter().find(|h| h.name == "stream").unwrap();
    assert_eq!(stream.kind, HandlerKind::Streaming);
}

#[tokio::test]
async fn test_tauri_server_concurrent_streams() {
    let mut router = Router::new();
    router.register_streaming_with_args("echo", |args: serde_json::Value, tx: StreamSender| async move {
        let tag = args["tag"].as_str().unwrap_or("?").to_string();
        tx.send(format!("{tag}-1")).await.ok();
        tx.send(format!("{tag}-2")).await.ok();
        format!("done-{tag}")
    });

    let server = TauriServer::new(router);

    let (mut rx_a, handle_a) = server
        .call_streaming_handler("echo", r#"{"tag":"A"}"#)
        .unwrap();
    let (mut rx_b, handle_b) = server
        .call_streaming_handler("echo", r#"{"tag":"B"}"#)
        .unwrap();

    let (res_a, res_b) = tokio::join!(handle_a, handle_b);

    assert_eq!(res_a.unwrap().unwrap().result, "done-A");
    assert_eq!(res_b.unwrap().unwrap().result, "done-B");

    assert_eq!(rx_a.recv().await, Some("A-1".to_string()));
    assert_eq!(rx_a.recv().await, Some("A-2".to_string()));
    assert_eq!(rx_b.recv().await, Some("B-1".to_string()));
    assert_eq!(rx_b.recv().await, Some("B-2".to_string()));
}

#[tokio::test]
async fn test_tauri_server_cancel_stream() {
    let mut router = Router::new();
    router.register_streaming("slow", |tx: StreamSender| async move {
        for i in 0..1000 {
            if tx.send(format!("{i}")).await.is_err() {
                return format!("cancelled at {i}");
            }
            tokio::task::yield_now().await;
        }
        "completed".to_string()
    });

    let server = TauriServer::new(router);
    let (rx, handle) = server.call_streaming_handler("slow", "{}").unwrap();

    // Drop the receiver to cancel
    drop(rx);

    let result = handle.await.unwrap().unwrap();
    // Handler should have stopped before reaching 1000
    assert!(result.result.starts_with("cancelled at"));
}
