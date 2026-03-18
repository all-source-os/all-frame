//! Integration tests for Router streaming registration and dispatch

use allframe_core::router::{Router, State, StreamSender};
use std::sync::Arc;

#[tokio::test]
async fn test_register_and_call_streaming_handler() {
    let mut router = Router::new();
    router.register_streaming("stream", |tx: StreamSender| async move {
        for i in 0..5 {
            tx.send(format!("item-{i}")).await.ok();
        }
        "done".to_string()
    });

    let router = Arc::new(router);
    let (mut rx, handle) = router.spawn_streaming_handler("stream", "{}").unwrap();

    let mut items = Vec::new();
    while let Some(item) = rx.recv().await {
        items.push(item);
    }

    let result = handle.await.unwrap();
    assert_eq!(result, Ok("done".to_string()));
    assert_eq!(items, vec!["item-0", "item-1", "item-2", "item-3", "item-4"]);
}

#[tokio::test]
async fn test_concurrent_streams_isolated() {
    let mut router = Router::new();
    router.register_streaming_with_args("echo", |args: serde_json::Value, tx: StreamSender| async move {
        let prefix = args["prefix"].as_str().unwrap_or("").to_string();
        for i in 0..3 {
            tx.send(format!("{prefix}-{i}")).await.ok();
        }
        "ok".to_string()
    });

    let router = Arc::new(router);

    let (mut rx1, h1) = router.spawn_streaming_handler("echo", r#"{"prefix":"A"}"#).unwrap();
    let (mut rx2, h2) = router.spawn_streaming_handler("echo", r#"{"prefix":"B"}"#).unwrap();

    let mut items1 = Vec::new();
    let mut items2 = Vec::new();

    let (r1, r2) = tokio::join!(
        async {
            while let Some(item) = rx1.recv().await { items1.push(item); }
            h1.await.unwrap()
        },
        async {
            while let Some(item) = rx2.recv().await { items2.push(item); }
            h2.await.unwrap()
        }
    );

    assert_eq!(r1, Ok("ok".to_string()));
    assert_eq!(r2, Ok("ok".to_string()));
    assert_eq!(items1, vec!["A-0", "A-1", "A-2"]);
    assert_eq!(items2, vec!["B-0", "B-1", "B-2"]);
}

#[tokio::test]
async fn test_cancellation_propagates() {
    let mut router = Router::new();
    router.register_streaming("infinite", |tx: StreamSender| async move {
        let mut i = 0;
        loop {
            if tx.is_closed() {
                break;
            }
            if tx.send(format!("{i}")).await.is_err() {
                break;
            }
            i += 1;
            tokio::task::yield_now().await;
        }
        format!("stopped at {i}")
    });

    let router = Arc::new(router);
    let (mut rx, handle) = router.spawn_streaming_handler("infinite", "{}").unwrap();

    // Read a few items
    let _ = rx.recv().await;
    let _ = rx.recv().await;

    // Drop receiver to cancel
    drop(rx);

    // Handler should complete (stop sending)
    let result = handle.await.unwrap();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cancellation_token_in_select() {
    let mut router = Router::new();
    router.register_streaming("token_cancel", |tx: StreamSender| async move {
        let token = tx.cancellation_token();
        tokio::select! {
            _ = token.cancelled() => {
                "cancelled".to_string()
            }
            _ = async {
                loop {
                    if tx.send("tick".to_string()).await.is_err() {
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                }
            } => {
                "stream ended".to_string()
            }
        }
    });

    let router = Arc::new(router);
    let (rx, handle) = router.spawn_streaming_handler("token_cancel", "{}").unwrap();

    // Drop receiver to trigger cancellation
    drop(rx);

    let result = handle.await.unwrap();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_stream_adapter_bridges_to_channel() {
    let mut router = Router::new();
    router.register_stream("items", || async {
        tokio_stream::iter(vec!["x".to_string(), "y".to_string(), "z".to_string()])
    });

    assert!(router.is_streaming("items"));

    let router = Arc::new(router);
    let (mut rx, handle) = router.spawn_streaming_handler("items", "{}").unwrap();

    let mut items = Vec::new();
    while let Some(item) = rx.recv().await {
        items.push(item);
    }
    let _ = handle.await;

    assert_eq!(items, vec!["x", "y", "z"]);
}

#[tokio::test]
async fn test_mixed_router_regular_and_streaming() {
    let mut router = Router::new();
    router.register("get_user", || async { r#"{"id":1}"#.to_string() });
    router.register_streaming("stream_updates", |tx: StreamSender| async move {
        tx.send("update-1".to_string()).await.ok();
        "done".to_string()
    });

    // Regular handler works
    let result = router.execute("get_user").await;
    assert_eq!(result, Ok(r#"{"id":1}"#.to_string()));
    assert!(!router.is_streaming("get_user"));
    assert!(router.is_streaming("stream_updates"));

    // Both appear in list_handlers
    let handlers = router.list_handlers();
    assert!(handlers.contains(&"get_user".to_string()));
    assert!(handlers.contains(&"stream_updates".to_string()));

    // Streaming handler works
    let router = Arc::new(router);
    let (mut rx, handle) = router.spawn_streaming_handler("stream_updates", "{}").unwrap();

    let mut items = Vec::new();
    while let Some(item) = rx.recv().await {
        items.push(item);
    }
    let result = handle.await.unwrap();

    assert_eq!(result, Ok("done".to_string()));
    assert_eq!(items, vec!["update-1"]);
}

#[tokio::test]
async fn test_streaming_with_state() {
    struct AppState {
        prefix: String,
    }

    #[derive(serde::Deserialize)]
    struct Input {
        name: String,
    }

    let mut router = Router::new().with_state(AppState {
        prefix: "Hello".to_string(),
    });
    router.register_streaming_with_state::<AppState, Input, _, _, _>(
        "greet_stream",
        |state: State<Arc<AppState>>, args: Input, tx: StreamSender| async move {
            tx.send(format!("{} {}", state.prefix, args.name))
                .await
                .ok();
            "greeted".to_string()
        },
    );

    let router = Arc::new(router);
    let (mut rx, handle) = router
        .spawn_streaming_handler("greet_stream", r#"{"name":"World"}"#)
        .unwrap();

    let item = rx.recv().await.unwrap();
    assert_eq!(item, "Hello World");

    let result = handle.await.unwrap();
    assert_eq!(result, Ok("greeted".to_string()));
}

#[tokio::test]
async fn test_backpressure_bounded_channel() {
    let mut router = Router::new();
    router.register_streaming("pressure", |tx: StreamSender| async move {
        for i in 0..100 {
            if tx.send(format!("{i}")).await.is_err() {
                return format!("stopped at {i}");
            }
        }
        "sent all 100".to_string()
    });

    let router = Arc::new(router);
    let (mut rx, handle) = router.spawn_streaming_handler("pressure", "{}").unwrap();

    // Consume all items concurrently
    let mut count = 0;
    while let Some(_) = rx.recv().await {
        count += 1;
    }

    let result = handle.await.unwrap();
    assert_eq!(result, Ok("sent all 100".to_string()));
    assert_eq!(count, 100);
}
