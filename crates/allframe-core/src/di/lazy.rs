//! Lazy initialization for DI containers
//!
//! Provides `LazyProvider<T>` for deferred initialization and `LazyContainer`
//! for bulk warm-up of lazy bindings.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use tokio::sync::OnceCell;

use super::DependencyError;

/// Type alias for an async factory that produces a value of type T.
type AsyncFactory<T> =
    dyn Fn() -> Pin<Box<dyn Future<Output = Result<T, DependencyError>> + Send>> + Send + Sync;

/// A provider that lazily initializes a value on first access.
///
/// Thread-safe: concurrent calls to `get()` will only trigger one initialization;
/// other callers wait for it to complete.
pub struct LazyProvider<T: Clone + Send + Sync + 'static> {
    cell: Arc<OnceCell<T>>,
    factory: Arc<AsyncFactory<T>>,
}

impl<T: Clone + Send + Sync + 'static> Clone for LazyProvider<T> {
    fn clone(&self) -> Self {
        Self {
            cell: Arc::clone(&self.cell),
            factory: Arc::clone(&self.factory),
        }
    }
}

impl<T: Clone + Send + Sync + 'static> LazyProvider<T> {
    /// Create a new lazy provider with the given async factory.
    pub fn new<F, Fut>(factory: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<T, DependencyError>> + Send + 'static,
    {
        Self {
            cell: Arc::new(OnceCell::new()),
            factory: Arc::new(move || Box::pin(factory())),
        }
    }

    /// Get the value, initializing it on first call.
    pub async fn get(&self) -> Result<T, DependencyError> {
        let factory = &self.factory;
        self.cell
            .get_or_try_init(|| factory())
            .await
            .cloned()
    }
}

/// Trait for type-erased lazy initialization (used by LazyContainer).
#[async_trait::async_trait]
trait LazyInit: Send + Sync {
    async fn init(&self) -> Result<(), DependencyError>;
}

struct LazyEntry<T: Clone + Send + Sync + 'static> {
    #[allow(dead_code)]
    name: String,
    provider: LazyProvider<T>,
}

#[async_trait::async_trait]
impl<T: Clone + Send + Sync + 'static> LazyInit for LazyEntry<T> {
    async fn init(&self) -> Result<(), DependencyError> {
        self.provider.get().await?;
        Ok(())
    }
}

/// Container that holds multiple lazy providers and can warm them up concurrently.
pub struct LazyContainer {
    entries: Vec<Arc<dyn LazyInit>>,
}

impl Default for LazyContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl LazyContainer {
    /// Create a new empty container.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Register a lazy provider for type T with a name and factory.
    pub fn register_lazy<T, F, Fut>(&mut self, name: &str, factory: F)
    where
        T: Clone + Send + Sync + 'static,
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<T, DependencyError>> + Send + 'static,
    {
        let provider = LazyProvider::new(factory);
        self.entries.push(Arc::new(LazyEntry {
            name: name.to_string(),
            provider,
        }));
    }

    /// Initialize all lazy bindings concurrently.
    ///
    /// Spawns each initialization as a separate tokio task and waits for all
    /// to complete. If any initialization fails, the error is returned after
    /// all tasks finish.
    pub async fn warm_up(&self) -> Result<(), DependencyError> {
        let mut handles = Vec::with_capacity(self.entries.len());
        for entry in &self.entries {
            let entry = Arc::clone(entry);
            handles.push(tokio::spawn(async move { entry.init().await }));
        }
        for handle in handles {
            handle
                .await
                .map_err(|e| DependencyError::InitializationFailed {
                    name: "warm_up".to_string(),
                    source: Box::new(e),
                })??;
        }
        Ok(())
    }
}
