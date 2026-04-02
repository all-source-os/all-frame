# Erased Handler Registration Guide

> **TL;DR:** If you're registering 200+ handlers and hitting `E0275` overflow, switch from
> `router.register_result_with_args("name", handler)` to
> `router.register_erased("name", erase_handler_with_args!(handler, ArgsType))`.

## The Problem

Rust's trait solver has a finite recursion depth. Each generic handler registration
(e.g., `register_result_with_args<T, R, E, F, Fut>`) monomorphizes a 5-type-parameter
function. At ~300-640 handlers, the cumulative trait-resolution pressure overflows the
solver, producing errors like:

```
error[E0275]: overflow evaluating the requirement
    `&itertools::groupbylazy::ChunkBy<_, _, _>: IntoIterator`
```

The error appears on **unrelated** traits because the solver is exhausted globally, not
just for your handlers.

## The Solution

AllFrame provides a **non-generic registration path** via declarative macros that generate
fully concrete code. No generic function is monomorphized at the call site.

### Quick Migration

**Before:**
```rust
router.register("health", health_check);
router.register_with_args("get_user", get_user);
router.register_result_with_args("create_user", create_user);
router.register_result_with_state("update_user", update_user);
router.register_with_state_only("db_status", db_status);
router.register_streaming("events", event_stream);
```

**After (individual macros):**
```rust
use allframe_core::{
    erase_handler, erase_handler_with_args,
    erase_handler_with_state, erase_handler_with_state_only,
    erase_streaming_handler,
};

let states = router.shared_states();

router.register_erased("health", erase_handler!(health_check));
router.register_erased("get_user", erase_handler_with_args!(get_user, GetUserArgs));
router.register_erased("create_user", erase_handler_with_args!(create_user, CreateUserArgs));
router.register_erased("update_user", erase_handler_with_state!(update_user, AppState, UpdateArgs, states.clone()));
router.register_erased("db_status", erase_handler_with_state_only!(db_status, AppState, states.clone()));
router.register_streaming_erased("events", erase_streaming_handler!(event_stream));
```

**After (batch macro — recommended):**
```rust
use allframe_core::register_handlers_erased;

register_handlers_erased!(router, {
    "health"      => health_check(),
    "get_user"    => get_user(args: GetUserArgs),
    "create_user" => create_user(args: CreateUserArgs),
    "update_user" => update_user(state: AppState, args: UpdateArgs),
    "db_status"   => db_status(state: AppState),
    "events"      => stream event_stream(),
});
```

## Macro Reference

### Request/Response Handlers

#### `erase_handler!(handler)`
For handlers with no arguments: `async fn() -> R`

```rust
async fn health() -> String { "ok".into() }

router.register_erased("health", erase_handler!(health));
```

#### `erase_handler_with_args!(handler, ArgsType)`
For handlers with typed args: `async fn(T) -> R`

```rust
#[derive(Deserialize)]
struct GetUserArgs { id: u64 }

async fn get_user(args: GetUserArgs) -> Result<User, MyError> { ... }

router.register_erased("get_user", erase_handler_with_args!(get_user, GetUserArgs));
```

#### `erase_handler_with_state!(handler, StateType, ArgsType, states)`
For handlers with state injection + typed args: `async fn(State<Arc<S>>, T) -> R`

```rust
async fn update(state: State<Arc<AppState>>, args: UpdateArgs) -> Result<(), MyError> { ... }

let states = router.shared_states();
router.register_erased("update", erase_handler_with_state!(update, AppState, UpdateArgs, states));
```

#### `erase_handler_with_state_only!(handler, StateType, states)`
For handlers with state injection only: `async fn(State<Arc<S>>) -> R`

```rust
async fn db_status(state: State<Arc<AppState>>) -> String { ... }

let states = router.shared_states();
router.register_erased("db_status", erase_handler_with_state_only!(db_status, AppState, states));
```

### Streaming Handlers

The streaming macros mirror the request/response macros:

| Macro | Signature |
|-------|-----------|
| `erase_streaming_handler!(h)` | `async fn(StreamSender) -> R` |
| `erase_streaming_handler_with_args!(h, T)` | `async fn(T, StreamSender) -> R` |
| `erase_streaming_handler_with_state!(h, S, T, states)` | `async fn(State<Arc<S>>, T, StreamSender) -> R` |
| `erase_streaming_handler_with_state_only!(h, S, states)` | `async fn(State<Arc<S>>, StreamSender) -> R` |

### Batch Registration

`register_handlers_erased!` registers multiple handlers at once:

```rust
register_handlers_erased!(router, {
    // No args
    "health" => health(),

    // With typed args
    "get_user" => get_user(args: GetUserArgs),

    // With state + args
    "update" => update(state: AppState, args: UpdateArgs),

    // State only
    "db_status" => db_status(state: AppState),

    // Streaming variants use the `stream` keyword
    "events" => stream event_stream(),
    "feed"   => stream user_feed(args: FeedArgs),
    "live"   => stream live_query(state: AppState, args: QueryArgs),
    "ping"   => stream heartbeat(state: AppState),
});
```

The macro automatically calls `router.shared_states()` once and reuses it for all
state handlers in the batch.

## How It Works

The generic path:
```rust
// Compiler monomorphizes register_result_with_args<GetUserArgs, User, MyError, F, Fut>
// and ErasedHandler::with_args<F, GetUserArgs, Fut, Result<User, MyError>>
// → trait resolution for 5 abstract type parameters
router.register_result_with_args("get_user", get_user);
```

The erased path:
```rust
// erase_handler_with_args! generates concrete boxing code:
//   Box::new(move |args: &str| {
//       let parsed: Result<GetUserArgs, _> = serde_json::from_str(args);
//       match parsed { Ok(v) => { let fut = get_user(v); Box::pin(...) } ... }
//   })
// → trait resolution for concrete types only (GetUserArgs: DeserializeOwned)
router.register_erased("get_user", erase_handler_with_args!(get_user, GetUserArgs));
```

The difference: generic functions require the compiler to verify trait bounds for
**abstract** type parameters across all possible types. The erased macros generate
code where the compiler only checks that **specific concrete types** implement the
required traits. The latter is dramatically cheaper for the trait solver.

## When to Use Each Path

| Situation | Recommendation |
|-----------|----------------|
| < 200 handlers | Generic path (simpler, type inference) |
| 200-600 handlers | Either works; switch if you hit E0275 |
| > 600 handlers | Erased path (prevents E0275) |
| Hitting E0275 | Erased path (this is why it exists) |

You can mix both paths in the same router. Only the handlers registered via the
generic path contribute to trait-resolution pressure.

## FAQ

**Q: Is there a runtime cost?**
A: No. Both paths produce identical runtime code: a `Box<dyn Fn(&str) -> Pin<Box<Future>>>`.
The erased path adds zero overhead.

**Q: Can I still use `#[allframe_handler]`?**
A: Yes. The attribute macro validates signatures and suppresses dead-code warnings.
It's orthogonal to how you register the handler.

**Q: Why do I need to specify the args type explicitly?**
A: Declarative macros operate on tokens, not types. They can't infer `GetUserArgs` from
`get_user`'s signature. The explicit type tells the macro what to pass to `serde_json::from_str`.

**Q: What about handlers that return `Result<T, E>` vs plain `String`?**
A: Both work. The macros use `IntoHandlerResult::into_handler_result()` which handles
`String`, `Json<T>`, and `Result<T, E>` transparently.
