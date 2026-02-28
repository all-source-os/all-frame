//! Offline-aware resilience patterns
//!
//! Provides connectivity probing, offline circuit breakers, and
//! store-and-forward queuing for offline-first deployments.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

/// Connectivity status returned by a probe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectivityStatus {
    /// Fully connected
    Online,
    /// No connectivity
    Offline,
    /// Partial connectivity
    Degraded {
        /// Reason for degradation
        reason: String,
    },
}

/// Trait for checking network connectivity.
#[async_trait]
pub trait ConnectivityProbe: Send + Sync {
    /// Check current connectivity status.
    async fn check(&self) -> ConnectivityStatus;
}

/// Result of calling through an offline circuit breaker.
#[derive(Debug)]
pub enum CallResult<T, E> {
    /// The operation was executed (may have succeeded or failed).
    Executed(Result<T, E>),
    /// The operation was queued because connectivity is offline.
    Queued,
}

impl<T, E> CallResult<T, E> {
    /// Returns true if the operation was queued rather than executed.
    pub fn is_queued(&self) -> bool {
        matches!(self, CallResult::Queued)
    }
}

type BoxedFnOnce = Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send>;

/// Circuit breaker that queues operations when offline.
pub struct OfflineCircuitBreaker<P: ConnectivityProbe> {
    #[allow(dead_code)]
    name: String,
    probe: P,
    queue: Arc<Mutex<Vec<BoxedFnOnce>>>,
}

impl<P: ConnectivityProbe> OfflineCircuitBreaker<P> {
    /// Create a new offline circuit breaker.
    pub fn new(name: impl Into<String>, probe: P) -> Self {
        Self {
            name: name.into(),
            probe,
            queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Call a function, queuing it if offline.
    pub async fn call<F, Fut, T, E>(&self, f: F) -> CallResult<T, E>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Result<T, E>> + Send + 'static,
        T: Send + 'static,
        E: Send + 'static,
    {
        match self.probe.check().await {
            ConnectivityStatus::Online => {
                let result = f().await;
                CallResult::Executed(result)
            }
            _ => {
                // Queue the operation (fire-and-forget wrapper)
                let wrapper: BoxedFnOnce = Box::new(move || {
                    Box::pin(async move {
                        let _ = f().await;
                    })
                });
                self.queue.lock().await.push(wrapper);
                CallResult::Queued
            }
        }
    }

    /// Number of queued operations.
    pub async fn queued_count(&self) -> usize {
        self.queue.lock().await.len()
    }

    /// Drain and replay all queued operations.
    pub async fn drain(&self) -> Result<(), String> {
        let ops: Vec<BoxedFnOnce> = {
            let mut q = self.queue.lock().await;
            q.drain(..).collect()
        };
        for op in ops {
            op().await;
        }
        Ok(())
    }
}

/// A pending operation in a store-and-forward queue.
#[derive(Debug, Clone)]
pub struct PendingOperation {
    /// Unique identifier for this operation.
    pub id: String,
}

/// Report from replaying stored operations.
#[derive(Debug, Clone)]
pub struct ReplayReport {
    /// Number of operations successfully replayed.
    pub replayed: usize,
    /// Number of operations that failed during replay.
    pub failed: usize,
}

/// In-memory queue for store-and-forward operations.
#[derive(Clone)]
pub struct InMemoryQueue {
    ops: Arc<Mutex<Vec<PendingOperation>>>,
}

impl InMemoryQueue {
    /// Create a new empty queue.
    pub fn new() -> Self {
        Self {
            ops: Arc::new(Mutex::new(Vec::new())),
        }
    }

    async fn push(&self, op: PendingOperation) {
        self.ops.lock().await.push(op);
    }

    async fn drain_all(&self) -> Vec<PendingOperation> {
        let mut q = self.ops.lock().await;
        q.drain(..).collect()
    }

    async fn len(&self) -> usize {
        self.ops.lock().await.len()
    }

    async fn peek_all(&self) -> Vec<PendingOperation> {
        self.ops.lock().await.clone()
    }
}

impl Default for InMemoryQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Store-and-forward pattern: stores operations when offline, replays on reconnect.
pub struct StoreAndForward<Q = InMemoryQueue, P: ConnectivityProbe = AlwaysOnlineProbe> {
    queue: Q,
    #[allow(dead_code)]
    probe: P,
}

/// A probe that always reports online. Used as default.
pub struct AlwaysOnlineProbe;

#[async_trait]
impl ConnectivityProbe for AlwaysOnlineProbe {
    async fn check(&self) -> ConnectivityStatus {
        ConnectivityStatus::Online
    }
}

impl StoreAndForward<InMemoryQueue, AlwaysOnlineProbe> {
    /// Create a store-and-forward with in-memory queue and always-online probe.
    pub fn default_new() -> Self {
        Self {
            queue: InMemoryQueue::new(),
            probe: AlwaysOnlineProbe,
        }
    }
}

impl<P: ConnectivityProbe> StoreAndForward<InMemoryQueue, P> {
    /// Create a new store-and-forward with the given queue and probe.
    pub fn new(queue: InMemoryQueue, probe: P) -> Self {
        Self { queue, probe }
    }

    /// Execute an operation; if it fails, store it for later replay.
    pub async fn execute<F, Fut>(&self, id: &str, f: F)
    where
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = Result<(), String>> + Send,
    {
        let result = f().await;
        if result.is_err() {
            self.queue
                .push(PendingOperation { id: id.to_string() })
                .await;
        }
    }

    /// Number of pending operations.
    pub async fn pending_count(&self) -> usize {
        self.queue.len().await
    }

    /// Peek at all pending operations (FIFO order).
    pub async fn peek_pending(&self) -> Vec<PendingOperation> {
        self.queue.peek_all().await
    }

    /// Replay all pending operations through the given handler.
    pub async fn replay_all<F, Fut>(&self, handler: F) -> Result<ReplayReport, String>
    where
        F: Fn(String) -> Fut + Send,
        Fut: Future<Output = Result<(), String>> + Send,
    {
        let ops = self.queue.drain_all().await;
        let mut replayed = 0;
        let mut failed = 0;
        for op in ops {
            match handler(op.id).await {
                Ok(()) => replayed += 1,
                Err(_) => failed += 1,
            }
        }
        Ok(ReplayReport { replayed, failed })
    }
}
