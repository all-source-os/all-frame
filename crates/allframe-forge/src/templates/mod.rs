//! Template modules for project scaffolding
//!
//! This module contains all the template strings used when generating
//! new AllFrame projects with the `allframe ignite` command.
//!
//! Templates are organized by archetype:
//! - `basic`: Simple Clean Architecture project (default)
//! - `gateway`: API Gateway service with gRPC, resilience, and caching
//! - `consumer`: Event consumer service with Kafka, idempotency, and DLQ
//! - `producer`: Event producer service with outbox pattern and transactional messaging
//! - `bff`: Backend for Frontend API aggregation service
//! - `scheduled`: Scheduled jobs service with cron-based task execution
//! - `websocket`: WebSocket gateway for real-time bidirectional communication
//! - `saga`: Saga orchestrator for distributed transaction coordination
//! - `acl`/`legacy-adapter`: Legacy system adapter (anti-corruption layer)

pub mod acl;
pub mod basic;
pub mod bff;
pub mod consumer;
pub mod gateway;
pub mod producer;
pub mod saga;
pub mod scheduled;
pub mod websocket;

pub use basic::*;
