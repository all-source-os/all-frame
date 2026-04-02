// ─── Non-generic handler erasure macros ────────────────────────────────────
//
// These macros generate **fully concrete** boxing code — no generic function
// is monomorphized at the call site. This keeps trait-resolution pressure
// near zero even with hundreds of handlers, preventing E0275 overflow.
//
// See ADR-0005 for the design rationale.

/// Erase a **no-args** handler into an [`ErasedHandler`](crate::router::ErasedHandler).
///
/// The handler must be an `async fn() -> R` where `R: IntoHandlerResult`.
///
/// # Example
///
/// ```rust,ignore
/// use allframe_core::{erase_handler, router::Router};
///
/// #[allframe_macros::allframe_handler]
/// async fn health() -> String { "ok".into() }
///
/// let mut router = Router::new();
/// router.register_erased("health", erase_handler!(health));
/// ```
#[macro_export]
macro_rules! erase_handler {
    ($handler:expr) => {
        $crate::router::ErasedHandler::from_closure(Box::new(move |_args: &str| {
            let fut = $handler();
            Box::pin(async move {
                <_ as $crate::router::IntoHandlerResult>::into_handler_result(fut.await)
            })
                as ::std::pin::Pin<
                    Box<dyn ::std::future::Future<Output = ::std::result::Result<String, String>> + Send>,
                >
        }))
    };
}

/// Erase a handler **with typed args** into an [`ErasedHandler`](crate::router::ErasedHandler).
///
/// The handler must be an `async fn(T) -> R` where `T: DeserializeOwned` and
/// `R: IntoHandlerResult`.
///
/// # Example
///
/// ```rust,ignore
/// use allframe_core::{erase_handler_with_args, router::Router};
///
/// #[allframe_macros::allframe_handler]
/// async fn get_user(args: GetUserArgs) -> Result<User, MyError> { todo!() }
///
/// let mut router = Router::new();
/// router.register_erased(
///     "get_user",
///     erase_handler_with_args!(get_user, GetUserArgs),
/// );
/// ```
#[macro_export]
macro_rules! erase_handler_with_args {
    ($handler:expr, $args_ty:ty) => {
        $crate::router::ErasedHandler::from_closure(Box::new(move |args_str: &str| {
            let parsed: ::std::result::Result<$args_ty, _> = ::serde_json::from_str(args_str);
            match parsed {
                Ok(value) => {
                    let fut = $handler(value);
                    Box::pin(async move {
                        <_ as $crate::router::IntoHandlerResult>::into_handler_result(fut.await)
                    })
                        as ::std::pin::Pin<
                            Box<
                                dyn ::std::future::Future<
                                        Output = ::std::result::Result<String, String>,
                                    > + Send,
                            >,
                        >
                }
                Err(e) => Box::pin(async move {
                    Err(format!("Failed to deserialize args: {e}"))
                })
                    as ::std::pin::Pin<
                        Box<
                            dyn ::std::future::Future<
                                    Output = ::std::result::Result<String, String>,
                                > + Send,
                        >,
                    >,
            }
        }))
    };
}

/// Erase a handler **with state + typed args** into an [`ErasedHandler`](crate::router::ErasedHandler).
///
/// The handler must be an `async fn(State<Arc<S>>, T) -> R`.
///
/// # Example
///
/// ```rust,ignore
/// use allframe_core::{erase_handler_with_state, router::Router};
///
/// #[allframe_macros::allframe_handler]
/// async fn get_user(
///     state: State<Arc<AppState>>,
///     args: GetUserArgs,
/// ) -> Result<User, MyError> { todo!() }
///
/// let mut router = Router::new();
/// let states = router.shared_states();
/// router.register_erased(
///     "get_user",
///     erase_handler_with_state!(get_user, AppState, GetUserArgs, states),
/// );
/// ```
#[macro_export]
macro_rules! erase_handler_with_state {
    ($handler:expr, $state_ty:ty, $args_ty:ty, $states:expr) => {{
        let states = $states;
        $crate::router::ErasedHandler::from_closure(Box::new(move |args_str: &str| {
            let state_arc: ::std::result::Result<::std::sync::Arc<$state_ty>, String> =
                $crate::router::resolve_state(&states);
            match state_arc {
                Ok(s) => {
                    let parsed: ::std::result::Result<$args_ty, _> =
                        ::serde_json::from_str(args_str);
                    match parsed {
                        Ok(value) => {
                            let fut = $handler($crate::router::State(s), value);
                            Box::pin(async move {
                                <_ as $crate::router::IntoHandlerResult>::into_handler_result(
                                    fut.await,
                                )
                            })
                                as ::std::pin::Pin<
                                    Box<
                                        dyn ::std::future::Future<
                                                Output = ::std::result::Result<String, String>,
                                            > + Send,
                                    >,
                                >
                        }
                        Err(e) => Box::pin(async move {
                            Err(format!("Failed to deserialize args: {e}"))
                        })
                            as ::std::pin::Pin<
                                Box<
                                    dyn ::std::future::Future<
                                            Output = ::std::result::Result<String, String>,
                                        > + Send,
                                    >,
                                >,
                    }
                }
                Err(msg) => {
                    Box::pin(async move { Err(msg) })
                        as ::std::pin::Pin<
                            Box<
                                dyn ::std::future::Future<
                                        Output = ::std::result::Result<String, String>,
                                    > + Send,
                            >,
                        >
                }
            }
        }))
    }};
}

/// Erase a **state-only** handler into an [`ErasedHandler`](crate::router::ErasedHandler).
///
/// The handler must be an `async fn(State<Arc<S>>) -> R`.
///
/// # Example
///
/// ```rust,ignore
/// use allframe_core::{erase_handler_with_state_only, router::Router};
///
/// #[allframe_macros::allframe_handler]
/// async fn db_status(state: State<Arc<AppState>>) -> String { todo!() }
///
/// let mut router = Router::new();
/// let states = router.shared_states();
/// router.register_erased(
///     "db_status",
///     erase_handler_with_state_only!(db_status, AppState, states),
/// );
/// ```
#[macro_export]
macro_rules! erase_handler_with_state_only {
    ($handler:expr, $state_ty:ty, $states:expr) => {{
        let states = $states;
        $crate::router::ErasedHandler::from_closure(Box::new(move |_args: &str| {
            let state_arc: ::std::result::Result<::std::sync::Arc<$state_ty>, String> =
                $crate::router::resolve_state(&states);
            match state_arc {
                Ok(s) => {
                    let fut = $handler($crate::router::State(s));
                    Box::pin(async move {
                        <_ as $crate::router::IntoHandlerResult>::into_handler_result(fut.await)
                    })
                        as ::std::pin::Pin<
                            Box<
                                dyn ::std::future::Future<
                                        Output = ::std::result::Result<String, String>,
                                    > + Send,
                            >,
                        >
                }
                Err(msg) => {
                    Box::pin(async move { Err(msg) })
                        as ::std::pin::Pin<
                            Box<
                                dyn ::std::future::Future<
                                        Output = ::std::result::Result<String, String>,
                                    > + Send,
                            >,
                        >
                }
            }
        }))
    }};
}

// ─── Streaming handler erasure macros ──────────────────────────────────────

/// Erase a **no-args streaming** handler into an [`ErasedStreamHandler`](crate::router::ErasedStreamHandler).
///
/// The handler must be an `async fn(StreamSender) -> R`.
///
/// # Example
///
/// ```rust,ignore
/// use allframe_core::{erase_streaming_handler, router::{Router, StreamSender}};
///
/// #[allframe_macros::allframe_handler(streaming)]
/// async fn stream_data(tx: StreamSender) -> String { "done".into() }
///
/// let mut router = Router::new();
/// router.register_streaming_erased("stream_data", erase_streaming_handler!(stream_data));
/// ```
#[macro_export]
macro_rules! erase_streaming_handler {
    ($handler:expr) => {
        $crate::router::ErasedStreamHandler::from_closure(Box::new(
            move |_args: &str, tx: $crate::router::StreamSender| {
                let fut = $handler(tx);
                Box::pin(async move {
                    <_ as $crate::router::IntoHandlerResult>::into_handler_result(fut.await)
                })
                    as ::std::pin::Pin<
                        Box<
                            dyn ::std::future::Future<
                                    Output = ::std::result::Result<String, String>,
                                > + Send,
                        >,
                    >
            },
        ))
    };
}

/// Erase a streaming handler **with typed args** into an [`ErasedStreamHandler`](crate::router::ErasedStreamHandler).
///
/// The handler must be an `async fn(T, StreamSender) -> R`.
///
/// # Example
///
/// ```rust,ignore
/// use allframe_core::{erase_streaming_handler_with_args, router::{Router, StreamSender}};
///
/// #[allframe_macros::allframe_handler(streaming)]
/// async fn stream_user(args: GetUserArgs, tx: StreamSender) -> String { "done".into() }
///
/// let mut router = Router::new();
/// router.register_streaming_erased(
///     "stream_user",
///     erase_streaming_handler_with_args!(stream_user, GetUserArgs),
/// );
/// ```
#[macro_export]
macro_rules! erase_streaming_handler_with_args {
    ($handler:expr, $args_ty:ty) => {
        $crate::router::ErasedStreamHandler::from_closure(Box::new(
            move |args_str: &str, tx: $crate::router::StreamSender| {
                let parsed: ::std::result::Result<$args_ty, _> = ::serde_json::from_str(args_str);
                match parsed {
                    Ok(value) => {
                        let fut = $handler(value, tx);
                        Box::pin(async move {
                            <_ as $crate::router::IntoHandlerResult>::into_handler_result(
                                fut.await,
                            )
                        })
                            as ::std::pin::Pin<
                                Box<
                                    dyn ::std::future::Future<
                                            Output = ::std::result::Result<String, String>,
                                        > + Send,
                                >,
                            >
                    }
                    Err(e) => Box::pin(async move {
                        Err(format!("Failed to deserialize args: {e}"))
                    })
                        as ::std::pin::Pin<
                            Box<
                                dyn ::std::future::Future<
                                        Output = ::std::result::Result<String, String>,
                                    > + Send,
                                >,
                            >,
                }
            },
        ))
    };
}

/// Erase a streaming handler **with state + typed args** into an [`ErasedStreamHandler`](crate::router::ErasedStreamHandler).
///
/// The handler must be an `async fn(State<Arc<S>>, T, StreamSender) -> R`.
#[macro_export]
macro_rules! erase_streaming_handler_with_state {
    ($handler:expr, $state_ty:ty, $args_ty:ty, $states:expr) => {{
        let states = $states;
        $crate::router::ErasedStreamHandler::from_closure(Box::new(
            move |args_str: &str, tx: $crate::router::StreamSender| {
                let state_arc: ::std::result::Result<::std::sync::Arc<$state_ty>, String> =
                    $crate::router::resolve_state(&states);
                match state_arc {
                    Ok(s) => {
                        let parsed: ::std::result::Result<$args_ty, _> =
                            ::serde_json::from_str(args_str);
                        match parsed {
                            Ok(value) => {
                                let fut = $handler($crate::router::State(s), value, tx);
                                Box::pin(async move {
                                    <_ as $crate::router::IntoHandlerResult>::into_handler_result(
                                        fut.await,
                                    )
                                })
                                    as ::std::pin::Pin<
                                        Box<
                                            dyn ::std::future::Future<
                                                    Output = ::std::result::Result<String, String>,
                                                > + Send,
                                        >,
                                    >
                            }
                            Err(e) => Box::pin(async move {
                                Err(format!("Failed to deserialize args: {e}"))
                            })
                                as ::std::pin::Pin<
                                    Box<
                                        dyn ::std::future::Future<
                                                Output = ::std::result::Result<String, String>,
                                            > + Send,
                                    >,
                                >,
                        }
                    }
                    Err(msg) => {
                        Box::pin(async move { Err(msg) })
                            as ::std::pin::Pin<
                                Box<
                                    dyn ::std::future::Future<
                                            Output = ::std::result::Result<String, String>,
                                        > + Send,
                                >,
                            >
                    }
                }
            },
        ))
    }};
}

/// Erase a streaming handler **with state only** into an [`ErasedStreamHandler`](crate::router::ErasedStreamHandler).
///
/// The handler must be an `async fn(State<Arc<S>>, StreamSender) -> R`.
#[macro_export]
macro_rules! erase_streaming_handler_with_state_only {
    ($handler:expr, $state_ty:ty, $states:expr) => {{
        let states = $states;
        $crate::router::ErasedStreamHandler::from_closure(Box::new(
            move |_args: &str, tx: $crate::router::StreamSender| {
                let state_arc: ::std::result::Result<::std::sync::Arc<$state_ty>, String> =
                    $crate::router::resolve_state(&states);
                match state_arc {
                    Ok(s) => {
                        let fut = $handler($crate::router::State(s), tx);
                        Box::pin(async move {
                            <_ as $crate::router::IntoHandlerResult>::into_handler_result(
                                fut.await,
                            )
                        })
                            as ::std::pin::Pin<
                                Box<
                                    dyn ::std::future::Future<
                                            Output = ::std::result::Result<String, String>,
                                        > + Send,
                                >,
                            >
                    }
                    Err(msg) => {
                        Box::pin(async move { Err(msg) })
                            as ::std::pin::Pin<
                                Box<
                                    dyn ::std::future::Future<
                                            Output = ::std::result::Result<String, String>,
                                        > + Send,
                                    >,
                                >
                    }
                }
            },
        ))
    }};
}

// ─── Batch registration macro ──────────────────────────────────────────────

/// Register multiple handlers at once using the **non-generic** erased path.
///
/// This is the recommended way to register large numbers of handlers.
/// Each entry generates **zero** generic monomorphizations, preventing
/// E0275 overflow regardless of handler count.
///
/// Compared to [`register_handlers!`] (which uses the generic registration
/// methods), this macro requires explicit type annotations but produces no
/// trait-resolution pressure.
///
/// # Syntax
///
/// ```rust,ignore
/// use allframe_core::{register_handlers_erased, router::Router};
///
/// let mut router = Router::new()
///     .with_state(app_state);
///
/// register_handlers_erased!(router, {
///     // No args
///     "health"      => health(),
///
///     // With typed args
///     "get_user"    => get_user(args: GetUserArgs),
///     "create_user" => create_user(args: CreateUserArgs),
///
///     // With state + args
///     "update_user" => update_user(state: AppState, args: UpdateArgs),
///
///     // State only
///     "db_status"   => db_status(state: AppState),
///
///     // Streaming (no args)
///     "events"      => stream events_stream(),
///
///     // Streaming with args
///     "user_feed"   => stream user_feed(args: FeedArgs),
///
///     // Streaming with state + args
///     "live_query"  => stream live_query(state: AppState, args: QueryArgs),
///
///     // Streaming with state only
///     "heartbeat"   => stream heartbeat(state: AppState),
/// });
/// ```
#[macro_export]
macro_rules! register_handlers_erased {
    ($router:expr, { $($entries:tt)* }) => {
        #[allow(unused_variables)]
        let states = $router.shared_states();
        $crate::__rhe_entries!($router, states, $($entries)*);
    };
}

/// Internal recursive TT muncher — do not use directly.
#[macro_export]
#[doc(hidden)]
macro_rules! __rhe_entries {
    // ── Terminal case ──────────────────────────────────────────────────
    ($router:expr, $states:expr, ) => {};

    // ── Streaming with state + args ────────────────────────────────────
    ($router:expr, $states:expr, $name:literal => stream $handler:ident (state: $sty:ty, args: $aty:ty), $($rest:tt)*) => {
        $router.register_streaming_erased(
            $name,
            $crate::erase_streaming_handler_with_state!($handler, $sty, $aty, $states.clone()),
        );
        $crate::__rhe_entries!($router, $states, $($rest)*)
    };
    // ── Streaming with state only ──────────────────────────────────────
    ($router:expr, $states:expr, $name:literal => stream $handler:ident (state: $sty:ty), $($rest:tt)*) => {
        $router.register_streaming_erased(
            $name,
            $crate::erase_streaming_handler_with_state_only!($handler, $sty, $states.clone()),
        );
        $crate::__rhe_entries!($router, $states, $($rest)*)
    };
    // ── Streaming with args ────────────────────────────────────────────
    ($router:expr, $states:expr, $name:literal => stream $handler:ident (args: $aty:ty), $($rest:tt)*) => {
        $router.register_streaming_erased(
            $name,
            $crate::erase_streaming_handler_with_args!($handler, $aty),
        );
        $crate::__rhe_entries!($router, $states, $($rest)*)
    };
    // ── Streaming no args ──────────────────────────────────────────────
    ($router:expr, $states:expr, $name:literal => stream $handler:ident (), $($rest:tt)*) => {
        $router.register_streaming_erased(
            $name,
            $crate::erase_streaming_handler!($handler),
        );
        $crate::__rhe_entries!($router, $states, $($rest)*)
    };
    // ── Request/response with state + args ─────────────────────────────
    ($router:expr, $states:expr, $name:literal => $handler:ident (state: $sty:ty, args: $aty:ty), $($rest:tt)*) => {
        $router.register_erased(
            $name,
            $crate::erase_handler_with_state!($handler, $sty, $aty, $states.clone()),
        );
        $crate::__rhe_entries!($router, $states, $($rest)*)
    };
    // ── Request/response with state only ───────────────────────────────
    ($router:expr, $states:expr, $name:literal => $handler:ident (state: $sty:ty), $($rest:tt)*) => {
        $router.register_erased(
            $name,
            $crate::erase_handler_with_state_only!($handler, $sty, $states.clone()),
        );
        $crate::__rhe_entries!($router, $states, $($rest)*)
    };
    // ── Request/response with args ─────────────────────────────────────
    ($router:expr, $states:expr, $name:literal => $handler:ident (args: $aty:ty), $($rest:tt)*) => {
        $router.register_erased(
            $name,
            $crate::erase_handler_with_args!($handler, $aty),
        );
        $crate::__rhe_entries!($router, $states, $($rest)*)
    };
    // ── Request/response no args ───────────────────────────────────────
    ($router:expr, $states:expr, $name:literal => $handler:ident (), $($rest:tt)*) => {
        $router.register_erased(
            $name,
            $crate::erase_handler!($handler),
        );
        $crate::__rhe_entries!($router, $states, $($rest)*)
    };
}
