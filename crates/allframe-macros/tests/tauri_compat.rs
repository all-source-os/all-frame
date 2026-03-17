use allframe_macros::tauri_compat;

// Test: basic function with multiple args
#[tauri_compat]
async fn greet(name: String, age: u32) -> String {
    format!(r#"{{"greeting":"Hello {}, age {}"}}"#, name, age)
}

#[tokio::test]
async fn test_greet_with_args_struct() {
    let args = GreetArgs {
        name: "Alice".to_string(),
        age: 30,
    };
    let result = greet(args).await;
    assert_eq!(result, r#"{"greeting":"Hello Alice, age 30"}"#);
}

#[tokio::test]
async fn test_greet_args_deserialize() {
    let args: GreetArgs = serde_json::from_str(r#"{"name":"Bob","age":25}"#).unwrap();
    assert_eq!(args.name, "Bob");
    assert_eq!(args.age, 25);
}

// Test: single arg
#[tauri_compat]
async fn echo(message: String) -> String {
    message
}

#[tokio::test]
async fn test_single_arg() {
    let args = EchoArgs {
        message: "hello".to_string(),
    };
    let result = echo(args).await;
    assert_eq!(result, "hello");
}

// Test: no args
#[tauri_compat]
async fn ping() -> String {
    "pong".to_string()
}

#[tokio::test]
async fn test_no_args() {
    let result = ping().await;
    assert_eq!(result, "pong");
}

// Test: Option args get #[serde(default)]
#[tauri_compat]
async fn optional_param(name: String, title: Option<String>) -> String {
    match title {
        Some(t) => format!("{} {}", t, name),
        None => name,
    }
}

#[tokio::test]
async fn test_option_with_value() {
    let args: OptionalParamArgs =
        serde_json::from_str(r#"{"name":"Alice","title":"Dr."}"#).unwrap();
    let result = optional_param(args).await;
    assert_eq!(result, "Dr. Alice");
}

#[tokio::test]
async fn test_option_omitted() {
    // title is omitted - should default to None thanks to #[serde(default)]
    let args: OptionalParamArgs = serde_json::from_str(r#"{"name":"Bob"}"#).unwrap();
    let result = optional_param(args).await;
    assert_eq!(result, "Bob");
}

// Test: multiple types
#[tauri_compat]
async fn multi_types(name: String, count: i32, enabled: bool, score: f64) -> String {
    format!("{} {} {} {}", name, count, enabled, score)
}

#[tokio::test]
async fn test_multi_types_deserialize() {
    let args: MultiTypesArgs =
        serde_json::from_str(r#"{"name":"test","count":5,"enabled":true,"score":9.5}"#).unwrap();
    assert_eq!(args.name, "test");
    assert_eq!(args.count, 5);
    assert!(args.enabled);
    assert!((args.score - 9.5).abs() < f64::EPSILON);
}

// Test: Vec args
#[tauri_compat]
async fn with_vec(items: Vec<String>, limit: u32) -> String {
    format!("{} items, limit {}", items.len(), limit)
}

#[tokio::test]
async fn test_vec_arg() {
    let args: WithVecArgs =
        serde_json::from_str(r#"{"items":["a","b","c"],"limit":10}"#).unwrap();
    let result = with_vec(args).await;
    assert_eq!(result, "3 items, limit 10");
}
