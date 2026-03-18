//! Integration tests for CQRS macros
//!
//! Verifies that #[command], #[query], and #[event] generate real trait
//! implementations that compile and satisfy allframe_core trait bounds.

use allframe_core::cqrs::{Command, Event, EventTypeName, Query};

// --- #[command] generates Command impl ---

#[allframe_macros::command]
struct CreateOrder {
    item: String,
    qty: u32,
}

#[test]
fn test_command_macro_generates_impl() {
    // This compiles only if Command is implemented for CreateOrder
    fn assert_command<T: Command>() {}
    assert_command::<CreateOrder>();
}

#[test]
fn test_command_struct_is_usable() {
    let cmd = CreateOrder {
        item: "widget".to_string(),
        qty: 5,
    };
    assert_eq!(cmd.item, "widget");
    assert_eq!(cmd.qty, 5);
}

// --- #[query] generates Query impl ---

#[allframe_macros::query]
struct GetOrderById {
    id: String,
}

#[test]
fn test_query_macro_generates_impl() {
    fn assert_query<T: Query>() {}
    assert_query::<GetOrderById>();
}

// --- #[event] generates EventTypeName + Event impls ---

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[allframe_macros::event]
enum OrderEvent {
    Created { id: String, item: String },
    Shipped { id: String },
}

#[test]
fn test_event_macro_generates_event_type_name() {
    let name = OrderEvent::event_type_name();
    assert_eq!(name, "OrderEvent");
}

#[test]
fn test_event_macro_generates_event_impl() {
    fn assert_event<T: Event>() {}
    assert_event::<OrderEvent>();
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[allframe_macros::event]
struct ItemCreated {
    id: String,
}

#[test]
fn test_event_macro_on_struct() {
    fn assert_event<T: Event>() {}
    assert_event::<ItemCreated>();
    assert_eq!(ItemCreated::event_type_name(), "ItemCreated");
}
