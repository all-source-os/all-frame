//! WebSocket Gateway archetype templates
//!
//! Templates for generating real-time WebSocket services with channel-based
//! messaging.

use crate::config::ProjectConfig;

/// Convert a string to PascalCase
fn to_pascal_case(s: &str) -> String {
    s.split(|c| c == '-' || c == '_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

/// Generate Cargo.toml for websocket gateway project
pub fn cargo_toml(config: &ProjectConfig) -> String {
    let ws = config.websocket_gateway.as_ref().unwrap();
    let name = &config.name;

    format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
description = "{display_name}"

[dependencies]
# AllFrame
allframe-core = {{ version = "0.1", features = ["resilience", "otel"] }}

# Web Framework & WebSocket
axum = {{ version = "0.7", features = ["ws"] }}
tokio-tungstenite = "0.26"
tower = "0.5"
tower-http = {{ version = "0.6", features = ["trace", "cors"] }}

# Async
tokio = {{ version = "1", features = ["full"] }}
async-trait = "0.1"
futures = "0.3"

# Serialization
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"

# Error handling
thiserror = "2.0"
anyhow = "1.0"

# Tracing & Metrics
tracing = "0.1"
tracing-subscriber = {{ version = "0.3", features = ["env-filter"] }}
opentelemetry = {{ version = "0.27", features = ["metrics"] }}

# Utilities
chrono = {{ version = "0.4", features = ["serde"] }}
uuid = {{ version = "1.0", features = ["v4", "serde"] }}
dashmap = "6.0"
dotenvy = "0.15"

[dev-dependencies]
tokio-test = "0.4"

[[bin]]
name = "{name}"
path = "src/main.rs"
"#,
        name = name,
        display_name = ws.display_name,
    )
}

/// Generate main.rs
pub fn main_rs(config: &ProjectConfig) -> String {
    let ws = config.websocket_gateway.as_ref().unwrap();
    let pascal_name = to_pascal_case(&ws.service_name);

    format!(
        r#"//! {display_name}
//!
//! A WebSocket gateway service for real-time bidirectional communication.

use std::sync::Arc;
use tracing::info;

mod config;
mod error;
mod domain;
mod application;
mod infrastructure;
mod presentation;

use config::Config;
use application::{pascal_name}Hub;
use infrastructure::HealthServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {{
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    // Load configuration
    let config = Config::from_env();
    info!("Starting {display_name}");
    info!("Configured channels: {{:?}}", config.channels.iter().map(|c| &c.name).collect::<Vec<_>>());

    // Create WebSocket hub
    let hub = Arc::new({pascal_name}Hub::new(config.clone()));

    // Start hub background tasks
    let hub_handle = {{
        let hub = hub.clone();
        tokio::spawn(async move {{
            hub.run().await
        }})
    }};

    // Start health server in background
    let health_port = config.server.health_port;
    let health_handle = tokio::spawn(async move {{
        let health_server = HealthServer::new(health_port);
        health_server.run().await
    }});

    // Create router and start WebSocket server
    let app = presentation::create_router(hub.clone());

    info!("Starting WebSocket server on port {{}}", config.server.http_port);
    let listener = tokio::net::TcpListener::bind(
        format!("0.0.0.0:{{}}", config.server.http_port)
    ).await?;
    axum::serve(listener, app).await?;

    hub_handle.abort();
    health_handle.abort();
    info!("WebSocket gateway shutdown complete");
    Ok(())
}}
"#,
        pascal_name = pascal_name,
        display_name = ws.display_name,
    )
}

/// Generate config.rs
pub fn config_rs(config: &ProjectConfig) -> String {
    let ws = config.websocket_gateway.as_ref().unwrap();

    let channel_configs: Vec<String> = ws
        .channels
        .iter()
        .map(|ch| {
            format!(
                r#"            ChannelConfig {{
                name: "{}".to_string(),
                description: "{}".to_string(),
                authenticated: {},
            }}"#,
                ch.name, ch.description, ch.authenticated
            )
        })
        .collect();

    format!(
        r#"//! Service configuration

use std::env;

/// Main configuration
#[derive(Debug, Clone)]
pub struct Config {{
    pub server: ServerConfig,
    pub channels: Vec<ChannelConfig>,
    pub max_connections_per_client: u32,
    pub heartbeat_interval_secs: u64,
    pub connection_timeout_secs: u64,
}}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {{
    pub http_port: u16,
    pub health_port: u16,
}}

/// Channel configuration
#[derive(Debug, Clone)]
pub struct ChannelConfig {{
    pub name: String,
    pub description: String,
    pub authenticated: bool,
}}

impl Config {{
    pub fn from_env() -> Self {{
        Self {{
            server: ServerConfig {{
                http_port: env::var("PORT")
                    .unwrap_or_else(|_| "{http_port}".to_string())
                    .parse()
                    .expect("PORT must be a number"),
                health_port: env::var("HEALTH_PORT")
                    .unwrap_or_else(|_| "{health_port}".to_string())
                    .parse()
                    .expect("HEALTH_PORT must be a number"),
            }},
            channels: vec![
{channel_configs}
            ],
            max_connections_per_client: env::var("MAX_CONNECTIONS_PER_CLIENT")
                .unwrap_or_else(|_| "{max_connections}".to_string())
                .parse()
                .expect("MAX_CONNECTIONS_PER_CLIENT must be a number"),
            heartbeat_interval_secs: env::var("HEARTBEAT_INTERVAL_SECS")
                .unwrap_or_else(|_| "{heartbeat_interval}".to_string())
                .parse()
                .expect("HEARTBEAT_INTERVAL_SECS must be a number"),
            connection_timeout_secs: env::var("CONNECTION_TIMEOUT_SECS")
                .unwrap_or_else(|_| "{connection_timeout}".to_string())
                .parse()
                .expect("CONNECTION_TIMEOUT_SECS must be a number"),
        }}
    }}
}}
"#,
        http_port = ws.server.http_port,
        health_port = ws.server.health_port,
        channel_configs = channel_configs.join(",\n"),
        max_connections = ws.max_connections_per_client,
        heartbeat_interval = ws.heartbeat_interval_secs,
        connection_timeout = ws.connection_timeout_secs,
    )
}

/// Generate error.rs
pub fn error_rs(config: &ProjectConfig) -> String {
    let ws = config.websocket_gateway.as_ref().unwrap();
    let pascal_name = to_pascal_case(&ws.service_name);

    format!(
        r#"//! Error types

use thiserror::Error;

/// WebSocket gateway errors
#[derive(Error, Debug)]
pub enum {pascal_name}Error {{
    #[error("Connection error: {{0}}")]
    Connection(String),

    #[error("Channel not found: {{0}}")]
    ChannelNotFound(String),

    #[error("Authentication required")]
    AuthRequired,

    #[error("Connection limit exceeded")]
    ConnectionLimitExceeded,

    #[error("Message send error: {{0}}")]
    SendError(String),

    #[error("Internal error: {{0}}")]
    Internal(String),
}}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate domain/mod.rs
pub fn domain_mod(_config: &ProjectConfig) -> String {
    r#"//! Domain layer

pub mod messages;
pub mod connection;

pub use messages::*;
pub use connection::*;
"#
    .to_string()
}

/// Generate domain/messages.rs
pub fn domain_messages(config: &ProjectConfig) -> String {
    let ws = config.websocket_gateway.as_ref().unwrap();
    let pascal_name = to_pascal_case(&ws.service_name);

    format!(
        r#"//! WebSocket message types

use chrono::{{DateTime, Utc}};
use serde::{{Deserialize, Serialize}};
use uuid::Uuid;

/// Client to server message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ClientMessage {{
    /// Subscribe to a channel
    Subscribe {{ channel: String }},
    /// Unsubscribe from a channel
    Unsubscribe {{ channel: String }},
    /// Send a message to a channel
    Publish {{ channel: String, data: serde_json::Value }},
    /// Ping for keepalive
    Ping,
}}

/// Server to client message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ServerMessage {{
    /// Subscription confirmed
    Subscribed {{ channel: String }},
    /// Unsubscription confirmed
    Unsubscribed {{ channel: String }},
    /// Message from a channel
    Message(ChannelMessage),
    /// Pong response
    Pong,
    /// Error message
    Error {{ code: String, message: String }},
}}

/// Channel message with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMessage {{
    pub id: Uuid,
    pub channel: String,
    pub data: serde_json::Value,
    pub sender_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
}}

impl ChannelMessage {{
    pub fn new(channel: String, data: serde_json::Value, sender_id: Option<Uuid>) -> Self {{
        Self {{
            id: Uuid::new_v4(),
            channel,
            data,
            sender_id,
            timestamp: Utc::now(),
        }}
    }}
}}

/// {pascal_name} event for internal use
#[derive(Debug, Clone)]
pub enum HubEvent {{
    ClientConnected {{ client_id: Uuid }},
    ClientDisconnected {{ client_id: Uuid }},
    Subscribe {{ client_id: Uuid, channel: String }},
    Unsubscribe {{ client_id: Uuid, channel: String }},
    Broadcast {{ channel: String, message: ChannelMessage }},
}}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate domain/connection.rs
pub fn domain_connection(_config: &ProjectConfig) -> String {
    r#"//! Connection management

use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashSet;

/// Represents a connected client
#[derive(Debug, Clone)]
pub struct ClientConnection {
    pub id: Uuid,
    pub connected_at: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
    pub subscriptions: HashSet<String>,
    pub user_id: Option<Uuid>,
}

impl ClientConnection {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            connected_at: now,
            last_heartbeat: now,
            subscriptions: HashSet::new(),
            user_id: None,
        }
    }

    pub fn with_user(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = Utc::now();
    }

    pub fn subscribe(&mut self, channel: &str) {
        self.subscriptions.insert(channel.to_string());
    }

    pub fn unsubscribe(&mut self, channel: &str) {
        self.subscriptions.remove(channel);
    }

    pub fn is_subscribed(&self, channel: &str) -> bool {
        self.subscriptions.contains(channel)
    }
}

impl Default for ClientConnection {
    fn default() -> Self {
        Self::new()
    }
}
"#
    .to_string()
}

/// Generate application/mod.rs
pub fn application_mod(_config: &ProjectConfig) -> String {
    r#"//! Application layer

pub mod hub;

pub use hub::*;
"#
    .to_string()
}

/// Generate application/hub.rs
pub fn application_hub(config: &ProjectConfig) -> String {
    let ws = config.websocket_gateway.as_ref().unwrap();
    let pascal_name = to_pascal_case(&ws.service_name);

    format!(
        r#"//! WebSocket hub for managing connections and channels

use std::sync::Arc;
use dashmap::DashMap;
use tokio::sync::{{broadcast, mpsc}};
use tracing::{{info, warn, debug}};
use uuid::Uuid;

use crate::config::Config;
use crate::domain::{{ClientConnection, ChannelMessage, HubEvent, ServerMessage}};
use crate::error::{pascal_name}Error;

/// Channel for sending messages to clients
pub type ClientSender = mpsc::UnboundedSender<ServerMessage>;

/// WebSocket hub managing all connections and channels
pub struct {pascal_name}Hub {{
    config: Config,
    /// Connected clients
    connections: DashMap<Uuid, ClientSender>,
    /// Channel subscriptions: channel_name -> set of client_ids
    subscriptions: DashMap<String, DashMap<Uuid, ()>>,
    /// Broadcast channel for hub events
    event_tx: broadcast::Sender<HubEvent>,
}}

impl {pascal_name}Hub {{
    pub fn new(config: Config) -> Self {{
        let (event_tx, _) = broadcast::channel(1024);

        // Pre-create channels from config
        let subscriptions = DashMap::new();
        for channel in &config.channels {{
            subscriptions.insert(channel.name.clone(), DashMap::new());
        }}

        Self {{
            config,
            connections: DashMap::new(),
            subscriptions,
            event_tx,
        }}
    }}

    /// Run the hub event loop
    pub async fn run(&self) {{
        let mut rx = self.event_tx.subscribe();

        loop {{
            match rx.recv().await {{
                Ok(event) => {{
                    self.handle_event(event).await;
                }}
                Err(broadcast::error::RecvError::Lagged(n)) => {{
                    warn!("Hub event receiver lagged by {{}} messages", n);
                }}
                Err(broadcast::error::RecvError::Closed) => {{
                    info!("Hub event channel closed");
                    break;
                }}
            }}
        }}
    }}

    async fn handle_event(&self, event: HubEvent) {{
        match event {{
            HubEvent::ClientConnected {{ client_id }} => {{
                debug!(client_id = %client_id, "Client connected");
            }}
            HubEvent::ClientDisconnected {{ client_id }} => {{
                debug!(client_id = %client_id, "Client disconnected");
                self.cleanup_client(client_id);
            }}
            HubEvent::Subscribe {{ client_id, channel }} => {{
                if let Some(subs) = self.subscriptions.get(&channel) {{
                    subs.insert(client_id, ());
                    debug!(client_id = %client_id, channel = %channel, "Client subscribed");
                }}
            }}
            HubEvent::Unsubscribe {{ client_id, channel }} => {{
                if let Some(subs) = self.subscriptions.get(&channel) {{
                    subs.remove(&client_id);
                    debug!(client_id = %client_id, channel = %channel, "Client unsubscribed");
                }}
            }}
            HubEvent::Broadcast {{ channel, message }} => {{
                self.broadcast_to_channel(&channel, message).await;
            }}
        }}
    }}

    fn cleanup_client(&self, client_id: Uuid) {{
        self.connections.remove(&client_id);
        for entry in self.subscriptions.iter() {{
            entry.value().remove(&client_id);
        }}
    }}

    async fn broadcast_to_channel(&self, channel: &str, message: ChannelMessage) {{
        if let Some(subscribers) = self.subscriptions.get(channel) {{
            let msg = ServerMessage::Message(message);
            for entry in subscribers.iter() {{
                let client_id = *entry.key();
                if let Some(sender) = self.connections.get(&client_id) {{
                    let _ = sender.send(msg.clone());
                }}
            }}
        }}
    }}

    /// Register a new client connection
    pub fn register_client(&self, client_id: Uuid, sender: ClientSender) {{
        self.connections.insert(client_id, sender);
        let _ = self.event_tx.send(HubEvent::ClientConnected {{ client_id }});
    }}

    /// Remove a client connection
    pub fn unregister_client(&self, client_id: Uuid) {{
        let _ = self.event_tx.send(HubEvent::ClientDisconnected {{ client_id }});
    }}

    /// Subscribe client to a channel
    pub fn subscribe(&self, client_id: Uuid, channel: &str) -> Result<(), {pascal_name}Error> {{
        if !self.subscriptions.contains_key(channel) {{
            return Err({pascal_name}Error::ChannelNotFound(channel.to_string()));
        }}

        let _ = self.event_tx.send(HubEvent::Subscribe {{
            client_id,
            channel: channel.to_string(),
        }});

        // Send confirmation
        if let Some(sender) = self.connections.get(&client_id) {{
            let _ = sender.send(ServerMessage::Subscribed {{
                channel: channel.to_string(),
            }});
        }}

        Ok(())
    }}

    /// Unsubscribe client from a channel
    pub fn unsubscribe(&self, client_id: Uuid, channel: &str) {{
        let _ = self.event_tx.send(HubEvent::Unsubscribe {{
            client_id,
            channel: channel.to_string(),
        }});

        if let Some(sender) = self.connections.get(&client_id) {{
            let _ = sender.send(ServerMessage::Unsubscribed {{
                channel: channel.to_string(),
            }});
        }}
    }}

    /// Publish a message to a channel
    pub fn publish(&self, channel: &str, data: serde_json::Value, sender_id: Option<Uuid>) -> Result<(), {pascal_name}Error> {{
        if !self.subscriptions.contains_key(channel) {{
            return Err({pascal_name}Error::ChannelNotFound(channel.to_string()));
        }}

        let message = ChannelMessage::new(channel.to_string(), data, sender_id);
        let _ = self.event_tx.send(HubEvent::Broadcast {{
            channel: channel.to_string(),
            message,
        }});

        Ok(())
    }}

    /// Send pong response to a client
    pub fn send_pong(&self, client_id: Uuid) {{
        if let Some(sender) = self.connections.get(&client_id) {{
            let _ = sender.send(ServerMessage::Pong);
        }}
    }}

    /// Get the number of connected clients
    pub fn connection_count(&self) -> usize {{
        self.connections.len()
    }}

    /// Get available channels
    pub fn channels(&self) -> Vec<String> {{
        self.config.channels.iter().map(|c| c.name.clone()).collect()
    }}
}}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate infrastructure/mod.rs
pub fn infrastructure_mod(_config: &ProjectConfig) -> String {
    r#"//! Infrastructure layer

pub mod health;

pub use health::*;
"#
    .to_string()
}

/// Generate infrastructure/health.rs
pub fn infrastructure_health(_config: &ProjectConfig) -> String {
    r#"//! Health check server

use axum::{routing::get, Router};
use tracing::info;

/// Health check server for Kubernetes probes
pub struct HealthServer {
    port: u16,
}

impl HealthServer {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub async fn run(&self) -> Result<(), std::io::Error> {
        let app = Router::new()
            .route("/health", get(health))
            .route("/ready", get(ready));

        let addr = format!("0.0.0.0:{}", self.port);
        info!("Health server listening on {}", addr);

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await
    }
}

async fn health() -> &'static str {
    "OK"
}

async fn ready() -> &'static str {
    "OK"
}
"#
    .to_string()
}

/// Generate presentation/mod.rs
pub fn presentation_mod(_config: &ProjectConfig) -> String {
    r#"//! Presentation layer

pub mod handlers;

pub use handlers::*;
"#
    .to_string()
}

/// Generate presentation/handlers.rs
pub fn presentation_handlers(config: &ProjectConfig) -> String {
    let ws = config.websocket_gateway.as_ref().unwrap();
    let pascal_name = to_pascal_case(&ws.service_name);

    format!(
        r#"//! WebSocket handlers

use std::sync::Arc;
use axum::{{
    extract::{{ws::{{Message, WebSocket, WebSocketUpgrade}}, State}},
    response::IntoResponse,
    routing::get,
    Router,
}};
use futures::{{SinkExt, StreamExt}};
use tokio::sync::mpsc;
use tracing::{{error, info, debug}};
use uuid::Uuid;

use crate::application::{pascal_name}Hub;
use crate::domain::{{ClientConnection, ClientMessage, ServerMessage}};

type AppState = Arc<{pascal_name}Hub>;

/// Create the WebSocket router
pub fn create_router(hub: Arc<{pascal_name}Hub>) -> Router {{
    Router::new()
        .route("/ws", get(ws_handler))
        .route("/channels", get(list_channels))
        .route("/stats", get(stats))
        .with_state(hub)
}}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(hub): State<AppState>,
) -> impl IntoResponse {{
    ws.on_upgrade(move |socket| handle_socket(socket, hub))
}}

async fn handle_socket(socket: WebSocket, hub: Arc<{pascal_name}Hub>) {{
    let connection = ClientConnection::new();
    let client_id = connection.id;

    info!(client_id = %client_id, "WebSocket connection established");

    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Create channel for sending messages to this client
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMessage>();

    // Register client with hub
    hub.register_client(client_id, tx);

    // Spawn task to forward messages from hub to WebSocket
    let send_task = tokio::spawn(async move {{
        while let Some(msg) = rx.recv().await {{
            match serde_json::to_string(&msg) {{
                Ok(text) => {{
                    if ws_sender.send(Message::Text(text.into())).await.is_err() {{
                        break;
                    }}
                }}
                Err(e) => {{
                    error!("Failed to serialize message: {{}}", e);
                }}
            }}
        }}
    }});

    // Process incoming messages
    while let Some(result) = ws_receiver.next().await {{
        match result {{
            Ok(Message::Text(text)) => {{
                match serde_json::from_str::<ClientMessage>(&text) {{
                    Ok(msg) => handle_client_message(msg, client_id, &hub),
                    Err(e) => {{
                        debug!("Invalid message format: {{}}", e);
                    }}
                }}
            }}
            Ok(Message::Close(_)) => {{
                break;
            }}
            Ok(Message::Ping(_)) => {{
                // Axum handles ping/pong automatically
            }}
            Ok(_) => {{
                // Ignore other message types
            }}
            Err(e) => {{
                error!(client_id = %client_id, error = %e, "WebSocket error");
                break;
            }}
        }}
    }}

    // Cleanup
    hub.unregister_client(client_id);
    send_task.abort();

    info!(client_id = %client_id, "WebSocket connection closed");
}}

fn handle_client_message(msg: ClientMessage, client_id: Uuid, hub: &{pascal_name}Hub) {{
    match msg {{
        ClientMessage::Subscribe {{ channel }} => {{
            if let Err(e) = hub.subscribe(client_id, &channel) {{
                error!(client_id = %client_id, error = %e, "Subscribe failed");
            }}
        }}
        ClientMessage::Unsubscribe {{ channel }} => {{
            hub.unsubscribe(client_id, &channel);
        }}
        ClientMessage::Publish {{ channel, data }} => {{
            if let Err(e) = hub.publish(&channel, data, Some(client_id)) {{
                error!(client_id = %client_id, error = %e, "Publish failed");
            }}
        }}
        ClientMessage::Ping => {{
            hub.send_pong(client_id);
        }}
    }}
}}

async fn list_channels(State(hub): State<AppState>) -> axum::Json<Vec<String>> {{
    axum::Json(hub.channels())
}}

async fn stats(State(hub): State<AppState>) -> axum::Json<serde_json::Value> {{
    axum::Json(serde_json::json!({{
        "connections": hub.connection_count(),
        "channels": hub.channels(),
    }}))
}}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate README.md
pub fn readme(config: &ProjectConfig) -> String {
    let ws = config.websocket_gateway.as_ref().unwrap();
    let name = &config.name;

    let channel_table: Vec<String> = ws
        .channels
        .iter()
        .map(|ch| {
            format!(
                "| {} | {} | {} |",
                ch.name,
                ch.description,
                if ch.authenticated { "Yes" } else { "No" }
            )
        })
        .collect();

    format!(
        r#"# {display_name}

A WebSocket gateway service built with AllFrame for real-time bidirectional communication.

## Features

- **Channel-based Messaging**: Subscribe/publish to named channels
- **Connection Management**: Automatic heartbeat and timeout handling
- **Scalable**: DashMap-based concurrent connection handling
- **OpenTelemetry**: Distributed tracing and metrics
- **Health Checks**: Kubernetes-ready liveness and readiness probes

## Prerequisites

- Rust 1.75+

## Configuration

Set the following environment variables:

```bash
# Server
PORT=8080
HEALTH_PORT=8081

# WebSocket
MAX_CONNECTIONS_PER_CLIENT=5
HEARTBEAT_INTERVAL_SECS=30
CONNECTION_TIMEOUT_SECS=60
```

## Channels

| Name | Description | Auth Required |
|------|-------------|---------------|
{channel_table}

## Running

```bash
# Development
cargo run

# Production
cargo build --release
./target/release/{name}
```

## WebSocket API

### Connect

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');
```

### Message Types

#### Subscribe to Channel
```json
{{"type": "Subscribe", "payload": {{"channel": "events"}}}}
```

#### Unsubscribe from Channel
```json
{{"type": "Unsubscribe", "payload": {{"channel": "events"}}}}
```

#### Publish to Channel
```json
{{"type": "Publish", "payload": {{"channel": "events", "data": {{"key": "value"}}}}}}
```

#### Ping
```json
{{"type": "Ping"}}
```

### Server Responses

```json
// Subscription confirmed
{{"type": "Subscribed", "payload": {{"channel": "events"}}}}

// Message received
{{"type": "Message", "payload": {{"id": "...", "channel": "events", "data": {{...}}, "timestamp": "..."}}}}

// Pong
{{"type": "Pong"}}

// Error
{{"type": "Error", "payload": {{"code": "CHANNEL_NOT_FOUND", "message": "..."}}}}
```

## REST Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | /ws | WebSocket upgrade endpoint |
| GET | /channels | List available channels |
| GET | /stats | Connection statistics |

## Architecture

```
┌───────────────────────────────────────────────────────────┐
│                   WebSocket Gateway                        │
│  ┌───────────┐    ┌──────────┐    ┌────────────────────┐  │
│  │  Handler  │───▶│   Hub    │───▶│   Subscriptions    │  │
│  └───────────┘    └──────────┘    └────────────────────┘  │
│        │               │                   │              │
│        ▼               ▼                   ▼              │
│  ┌───────────┐    ┌──────────┐    ┌────────────────────┐  │
│  │ WebSocket │    │ Broadcast│    │    Connections     │  │
│  │  Upgrade  │    │  Channel │    │      (DashMap)     │  │
│  └───────────┘    └──────────┘    └────────────────────┘  │
└───────────────────────────────────────────────────────────┘
```

## License

MIT
"#,
        display_name = ws.display_name,
        name = name,
        channel_table = channel_table.join("\n"),
    )
}

/// Generate Dockerfile
pub fn dockerfile(config: &ProjectConfig) -> String {
    let name = &config.name;

    format!(
        r#"FROM rust:1.75-slim as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy main to cache dependencies
RUN mkdir src && echo "fn main() {{}}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy source
COPY src ./src

# Build
RUN touch src/main.rs && cargo build --release

# Runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/{name} /app/

ENV PORT=8080
ENV HEALTH_PORT=8081
EXPOSE 8080 8081

CMD ["/app/{name}"]
"#,
        name = name,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("ws_gateway"), "WsGateway");
        assert_eq!(to_pascal_case("real-time"), "RealTime");
        assert_eq!(to_pascal_case("simple"), "Simple");
    }
}
