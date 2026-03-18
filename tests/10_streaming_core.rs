//! Integration tests for streaming core types: StreamSender, IntoStreamItem, StreamHandler

use allframe_core::router::{
    handler::StreamingHandlerFn, handler::StreamingHandlerWithArgs, IntoStreamItem, Json,
    StreamError, StreamHandler, StreamSender, DEFAULT_STREAM_CAPACITY,
};

// ─── StreamSender integration ───────────────────────────────────────────────

#[tokio::test]
async fn test_stream_sender_ordered_delivery() {
    let (tx, mut rx) = StreamSender::channel();

    for i in 0..5 {
        tx.send(format!("item-{i}")).await.unwrap();
    }
    drop(tx);

    for i in 0..5 {
        assert_eq!(rx.recv().await, Some(format!("item-{i}")));
    }
    assert_eq!(rx.recv().await, None);
}

#[tokio::test]
async fn test_stream_sender_json_items() {
    #[derive(serde::Serialize)]
    struct Token {
        text: String,
        index: usize,
    }

    let (tx, mut rx) = StreamSender::channel();

    tx.send(Json(Token {
        text: "hello".into(),
        index: 0,
    }))
    .await
    .unwrap();
    tx.send(Json(Token {
        text: "world".into(),
        index: 1,
    }))
    .await
    .unwrap();
    drop(tx);

    let item1 = rx.recv().await.unwrap();
    assert!(item1.contains("\"text\":\"hello\""));
    let item2 = rx.recv().await.unwrap();
    assert!(item2.contains("\"text\":\"world\""));
}

#[tokio::test]
async fn test_stream_sender_cancellation_token() {
    let (tx, rx) = StreamSender::channel();
    let token = tx.cancellation_token();

    assert!(!token.is_cancelled());
    tx.cancel();
    assert!(token.is_cancelled());

    // Channel still works after cancel (cancel is advisory)
    drop(rx);
    assert!(tx.is_closed());
}

#[tokio::test]
async fn test_stream_sender_cancel_future_resolves() {
    let (tx, _rx) = StreamSender::channel();
    let token = tx.cancellation_token();

    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        tx.cancel();
    });

    tokio::time::timeout(std::time::Duration::from_secs(1), token.cancelled())
        .await
        .expect("cancellation future should resolve");
}

#[tokio::test]
async fn test_stream_sender_closed_on_receiver_drop() {
    let (tx, rx) = StreamSender::channel();
    assert!(!tx.is_closed());
    drop(rx);
    assert!(tx.is_closed());
    assert_eq!(
        tx.send("late".to_string()).await,
        Err(StreamError::Closed)
    );
}

#[test]
fn test_into_stream_item_impls() {
    // String
    assert_eq!("hello".to_string().into_stream_item(), Ok("hello".to_string()));

    // Json
    assert_eq!(Json(42).into_stream_item(), Ok("42".to_string()));

    // Result Ok
    assert_eq!(Ok::<_, String>(99).into_stream_item(), Ok("99".to_string()));

    // Result Err
    assert_eq!(
        Err::<i32, _>("fail".to_string()).into_stream_item(),
        Err("fail".to_string())
    );
}

#[test]
fn test_default_stream_capacity() {
    assert_eq!(DEFAULT_STREAM_CAPACITY, 64);
}

// ─── StreamHandler trait integration ────────────────────────────────────────

#[tokio::test]
async fn test_streaming_handler_fn_sends_and_returns() {
    let handler = StreamingHandlerFn::new(|tx: StreamSender| async move {
        tx.send("a".to_string()).await.ok();
        tx.send("b".to_string()).await.ok();
        tx.send("c".to_string()).await.ok();
        "final-result".to_string()
    });

    let (tx, mut rx) = StreamSender::channel();
    let result = handler.call_streaming("{}", tx).await;

    assert_eq!(result, Ok("final-result".to_string()));
    assert_eq!(rx.recv().await, Some("a".to_string()));
    assert_eq!(rx.recv().await, Some("b".to_string()));
    assert_eq!(rx.recv().await, Some("c".to_string()));
}

#[tokio::test]
async fn test_streaming_handler_with_args_deserializes() {
    #[derive(serde::Deserialize)]
    struct Input {
        prefix: String,
        count: usize,
    }

    let handler = StreamingHandlerWithArgs::new(|args: Input, tx: StreamSender| async move {
        for i in 0..args.count {
            tx.send(format!("{}-{i}", args.prefix)).await.ok();
        }
        format!("sent {}", args.count)
    });

    let (tx, mut rx) = StreamSender::channel();
    let result = handler
        .call_streaming(r#"{"prefix":"msg","count":3}"#, tx)
        .await;

    assert_eq!(result, Ok("sent 3".to_string()));
    assert_eq!(rx.recv().await, Some("msg-0".to_string()));
    assert_eq!(rx.recv().await, Some("msg-1".to_string()));
    assert_eq!(rx.recv().await, Some("msg-2".to_string()));
}

#[tokio::test]
async fn test_streaming_handler_bad_args() {
    #[derive(serde::Deserialize)]
    struct Input {
        _x: i32,
    }

    let handler = StreamingHandlerWithArgs::new(|_: Input, _tx: StreamSender| async move {
        "unreachable".to_string()
    });

    let (tx, _rx) = StreamSender::channel();
    let result = handler.call_streaming("not-json", tx).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Failed to deserialize"));
}
