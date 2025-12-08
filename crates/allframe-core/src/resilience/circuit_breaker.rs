//! Circuit breaker pattern for fail-fast behavior.
//!
//! Prevents cascading failures by stopping requests to failing services.

use dashmap::DashMap;
use parking_lot::RwLock;
use std::future::Future;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Circuit breaker state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, requests flow through normally.
    Closed,
    /// Circuit is open, requests fail immediately.
    Open,
    /// Circuit is testing if the service has recovered.
    HalfOpen,
}

impl std::fmt::Display for CircuitState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitState::Closed => write!(f, "closed"),
            CircuitState::Open => write!(f, "open"),
            CircuitState::HalfOpen => write!(f, "half-open"),
        }
    }
}

/// Error returned when circuit is open.
#[derive(Debug, Clone)]
pub struct CircuitOpenError {
    /// Name of the circuit.
    pub circuit_name: String,
    /// Time until the circuit may close.
    pub retry_after: Duration,
}

impl std::fmt::Display for CircuitOpenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "circuit '{}' is open, retry after {:?}",
            self.circuit_name, self.retry_after
        )
    }
}

impl std::error::Error for CircuitOpenError {}

/// Configuration for circuit breaker behavior.
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening the circuit.
    pub failure_threshold: u32,
    /// Number of successes in half-open state to close the circuit.
    pub success_threshold: u32,
    /// How long the circuit stays open before transitioning to half-open.
    pub timeout: Duration,
    /// Window for counting failures (failures outside this window don't count).
    pub failure_window: Duration,
    /// Maximum concurrent requests in half-open state.
    pub half_open_requests: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(30),
            failure_window: Duration::from_secs(60),
            half_open_requests: 1,
        }
    }
}

impl CircuitBreakerConfig {
    /// Create a new config with specified failure threshold.
    pub fn new(failure_threshold: u32) -> Self {
        Self {
            failure_threshold,
            ..Default::default()
        }
    }

    /// Set the success threshold for closing the circuit.
    pub fn with_success_threshold(mut self, threshold: u32) -> Self {
        self.success_threshold = threshold;
        self
    }

    /// Set the timeout before transitioning from open to half-open.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the failure counting window.
    pub fn with_failure_window(mut self, window: Duration) -> Self {
        self.failure_window = window;
        self
    }

    /// Set the number of concurrent requests allowed in half-open state.
    pub fn with_half_open_requests(mut self, requests: u32) -> Self {
        self.half_open_requests = requests.max(1);
        self
    }
}

/// Statistics for a circuit breaker.
#[derive(Debug, Clone)]
pub struct CircuitBreakerStats {
    /// Current state.
    pub state: CircuitState,
    /// Total successful requests.
    pub success_count: u64,
    /// Total failed requests.
    pub failure_count: u64,
    /// Requests rejected due to open circuit.
    pub rejected_count: u64,
    /// Failures in current window.
    pub failures_in_window: u32,
    /// Successes in half-open state.
    pub half_open_successes: u32,
    /// Time since last state change.
    pub time_in_state: Duration,
}

/// Circuit breaker for a single resource.
pub struct CircuitBreaker {
    name: String,
    config: CircuitBreakerConfig,
    state: RwLock<CircuitState>,
    /// Timestamp of last state change.
    state_changed_at: RwLock<Instant>,
    /// Recent failures (timestamp).
    failures: RwLock<Vec<Instant>>,
    /// Successes in half-open state.
    half_open_successes: AtomicU32,
    /// Current half-open requests in flight.
    half_open_in_flight: AtomicU32,
    /// Statistics.
    success_count: AtomicU64,
    failure_count: AtomicU64,
    rejected_count: AtomicU64,
}

impl CircuitBreaker {
    /// Create a new circuit breaker.
    pub fn new(name: impl Into<String>, config: CircuitBreakerConfig) -> Self {
        Self {
            name: name.into(),
            config,
            state: RwLock::new(CircuitState::Closed),
            state_changed_at: RwLock::new(Instant::now()),
            failures: RwLock::new(Vec::new()),
            half_open_successes: AtomicU32::new(0),
            half_open_in_flight: AtomicU32::new(0),
            success_count: AtomicU64::new(0),
            failure_count: AtomicU64::new(0),
            rejected_count: AtomicU64::new(0),
        }
    }

    /// Get the circuit name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Check if a request can proceed.
    ///
    /// Returns `Ok(())` if the request can proceed, `Err(CircuitOpenError)` if the circuit is open.
    pub fn check(&self) -> Result<(), CircuitOpenError> {
        self.maybe_transition_to_half_open();

        let state = *self.state.read();

        match state {
            CircuitState::Closed => Ok(()),
            CircuitState::Open => {
                self.rejected_count.fetch_add(1, Ordering::Relaxed);
                let elapsed = self.state_changed_at.read().elapsed();
                let retry_after = self.config.timeout.saturating_sub(elapsed);
                Err(CircuitOpenError {
                    circuit_name: self.name.clone(),
                    retry_after,
                })
            }
            CircuitState::HalfOpen => {
                // Allow limited requests in half-open state
                let in_flight = self.half_open_in_flight.load(Ordering::Acquire);
                if in_flight < self.config.half_open_requests {
                    self.half_open_in_flight.fetch_add(1, Ordering::AcqRel);
                    Ok(())
                } else {
                    self.rejected_count.fetch_add(1, Ordering::Relaxed);
                    Err(CircuitOpenError {
                        circuit_name: self.name.clone(),
                        retry_after: Duration::from_millis(100),
                    })
                }
            }
        }
    }

    /// Record a successful request.
    pub fn record_success(&self) {
        self.success_count.fetch_add(1, Ordering::Relaxed);

        let state = *self.state.read();

        if state == CircuitState::HalfOpen {
            self.half_open_in_flight.fetch_sub(1, Ordering::AcqRel);
            let successes = self.half_open_successes.fetch_add(1, Ordering::AcqRel) + 1;

            if successes >= self.config.success_threshold {
                self.transition_to(CircuitState::Closed);
            }
        }
    }

    /// Record a failed request.
    pub fn record_failure(&self) {
        self.failure_count.fetch_add(1, Ordering::Relaxed);

        let state = *self.state.read();

        match state {
            CircuitState::Closed => {
                let now = Instant::now();
                let mut failures = self.failures.write();

                // Add new failure
                failures.push(now);

                // Remove old failures outside the window
                let cutoff = now - self.config.failure_window;
                failures.retain(|&t| t > cutoff);

                // Check threshold
                if failures.len() as u32 >= self.config.failure_threshold {
                    drop(failures); // Release lock before transition
                    self.transition_to(CircuitState::Open);
                }
            }
            CircuitState::HalfOpen => {
                self.half_open_in_flight.fetch_sub(1, Ordering::AcqRel);
                // Single failure in half-open reopens the circuit
                self.transition_to(CircuitState::Open);
            }
            CircuitState::Open => {
                // Already open, nothing to do
            }
        }
    }

    /// Execute an async operation through the circuit breaker.
    pub async fn call<F, Fut, T, E>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        self.check().map_err(CircuitBreakerError::CircuitOpen)?;

        match f().await {
            Ok(result) => {
                self.record_success();
                Ok(result)
            }
            Err(e) => {
                self.record_failure();
                Err(CircuitBreakerError::Inner(e))
            }
        }
    }

    /// Get the current state.
    pub fn get_state(&self) -> CircuitState {
        self.maybe_transition_to_half_open();
        *self.state.read()
    }

    /// Get statistics.
    pub fn get_stats(&self) -> CircuitBreakerStats {
        self.maybe_transition_to_half_open();

        let state = *self.state.read();
        let failures = self.failures.read();
        let now = Instant::now();
        let cutoff = now - self.config.failure_window;
        let failures_in_window = failures.iter().filter(|&&t| t > cutoff).count() as u32;

        CircuitBreakerStats {
            state,
            success_count: self.success_count.load(Ordering::Relaxed),
            failure_count: self.failure_count.load(Ordering::Relaxed),
            rejected_count: self.rejected_count.load(Ordering::Relaxed),
            failures_in_window,
            half_open_successes: self.half_open_successes.load(Ordering::Relaxed),
            time_in_state: self.state_changed_at.read().elapsed(),
        }
    }

    /// Reset the circuit breaker to closed state.
    pub fn reset(&self) {
        self.transition_to(CircuitState::Closed);
        self.failures.write().clear();
    }

    fn transition_to(&self, new_state: CircuitState) {
        let mut state = self.state.write();
        let old_state = *state;

        if old_state != new_state {
            *state = new_state;
            *self.state_changed_at.write() = Instant::now();

            // Reset half-open counters
            if new_state == CircuitState::HalfOpen || new_state == CircuitState::Closed {
                self.half_open_successes.store(0, Ordering::Relaxed);
                self.half_open_in_flight.store(0, Ordering::Relaxed);
            }

            // Clear failures when closing
            if new_state == CircuitState::Closed {
                self.failures.write().clear();
            }

            #[cfg(feature = "otel")]
            tracing::info!(
                circuit = %self.name,
                old_state = %old_state,
                new_state = %new_state,
                "circuit breaker state changed"
            );
        }
    }

    fn maybe_transition_to_half_open(&self) {
        let state = *self.state.read();
        if state == CircuitState::Open {
            let elapsed = self.state_changed_at.read().elapsed();
            if elapsed >= self.config.timeout {
                self.transition_to(CircuitState::HalfOpen);
            }
        }
    }
}

/// Error type for circuit breaker operations.
#[derive(Debug)]
pub enum CircuitBreakerError<E> {
    /// The circuit is open.
    CircuitOpen(CircuitOpenError),
    /// The inner operation failed.
    Inner(E),
}

impl<E: std::fmt::Display> std::fmt::Display for CircuitBreakerError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitBreakerError::CircuitOpen(e) => write!(f, "{}", e),
            CircuitBreakerError::Inner(e) => write!(f, "{}", e),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for CircuitBreakerError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CircuitBreakerError::CircuitOpen(e) => Some(e),
            CircuitBreakerError::Inner(e) => Some(e),
        }
    }
}

/// Manages multiple circuit breakers by name.
pub struct CircuitBreakerManager {
    breakers: DashMap<String, Arc<CircuitBreaker>>,
    default_config: CircuitBreakerConfig,
}

impl CircuitBreakerManager {
    /// Create a new manager with default configuration.
    pub fn new(default_config: CircuitBreakerConfig) -> Self {
        Self {
            breakers: DashMap::new(),
            default_config,
        }
    }

    /// Get or create a circuit breaker by name.
    pub fn get_or_create(&self, name: &str) -> Arc<CircuitBreaker> {
        self.breakers
            .entry(name.to_string())
            .or_insert_with(|| Arc::new(CircuitBreaker::new(name, self.default_config.clone())))
            .clone()
    }

    /// Get a circuit breaker if it exists.
    pub fn get(&self, name: &str) -> Option<Arc<CircuitBreaker>> {
        self.breakers.get(name).map(|r| r.clone())
    }

    /// Create a circuit breaker with custom config.
    pub fn create_with_config(&self, name: &str, config: CircuitBreakerConfig) -> Arc<CircuitBreaker> {
        let breaker = Arc::new(CircuitBreaker::new(name, config));
        self.breakers.insert(name.to_string(), breaker.clone());
        breaker
    }

    /// Get stats for all circuit breakers.
    pub fn get_all_stats(&self) -> Vec<(String, CircuitBreakerStats)> {
        self.breakers
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().get_stats()))
            .collect()
    }

    /// Reset all circuit breakers.
    pub fn reset_all(&self) {
        for entry in self.breakers.iter() {
            entry.value().reset();
        }
    }

    /// Remove a circuit breaker.
    pub fn remove(&self, name: &str) {
        self.breakers.remove(name);
    }

    /// Clear all circuit breakers.
    pub fn clear(&self) {
        self.breakers.clear();
    }

    /// Get the number of circuit breakers.
    pub fn len(&self) -> usize {
        self.breakers.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.breakers.is_empty()
    }
}

impl Default for CircuitBreakerManager {
    fn default() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_config_default() {
        let config = CircuitBreakerConfig::default();
        assert_eq!(config.failure_threshold, 5);
        assert_eq!(config.success_threshold, 3);
        assert_eq!(config.timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_circuit_breaker_config_builder() {
        let config = CircuitBreakerConfig::new(10)
            .with_success_threshold(5)
            .with_timeout(Duration::from_secs(60))
            .with_half_open_requests(3);

        assert_eq!(config.failure_threshold, 10);
        assert_eq!(config.success_threshold, 5);
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.half_open_requests, 3);
    }

    #[test]
    fn test_circuit_breaker_initial_state() {
        let cb = CircuitBreaker::new("test", CircuitBreakerConfig::default());
        assert_eq!(cb.get_state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_opens_on_failures() {
        let config = CircuitBreakerConfig::new(3);
        let cb = CircuitBreaker::new("test", config);

        // Three failures should open the circuit
        cb.record_failure();
        assert_eq!(cb.get_state(), CircuitState::Closed);
        cb.record_failure();
        assert_eq!(cb.get_state(), CircuitState::Closed);
        cb.record_failure();
        assert_eq!(cb.get_state(), CircuitState::Open);
    }

    #[test]
    fn test_circuit_breaker_check_when_open() {
        let config = CircuitBreakerConfig::new(1);
        let cb = CircuitBreaker::new("test", config);

        cb.record_failure();
        assert_eq!(cb.get_state(), CircuitState::Open);

        let result = cb.check();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.circuit_name, "test");
    }

    #[test]
    fn test_circuit_breaker_transitions_to_half_open() {
        let config = CircuitBreakerConfig::new(1).with_timeout(Duration::from_millis(10));
        let cb = CircuitBreaker::new("test", config);

        cb.record_failure();
        assert_eq!(cb.get_state(), CircuitState::Open);

        std::thread::sleep(Duration::from_millis(20));

        assert_eq!(cb.get_state(), CircuitState::HalfOpen);
    }

    #[test]
    fn test_circuit_breaker_closes_on_success() {
        let config = CircuitBreakerConfig::new(1)
            .with_timeout(Duration::from_millis(10))
            .with_success_threshold(2);
        let cb = CircuitBreaker::new("test", config);

        cb.record_failure();
        std::thread::sleep(Duration::from_millis(20));

        assert_eq!(cb.get_state(), CircuitState::HalfOpen);

        // Allow request in half-open
        cb.check().unwrap();
        cb.record_success();
        assert_eq!(cb.get_state(), CircuitState::HalfOpen);

        cb.check().unwrap();
        cb.record_success();
        assert_eq!(cb.get_state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_reopens_on_half_open_failure() {
        let config = CircuitBreakerConfig::new(1).with_timeout(Duration::from_millis(10));
        let cb = CircuitBreaker::new("test", config);

        cb.record_failure();
        std::thread::sleep(Duration::from_millis(20));

        assert_eq!(cb.get_state(), CircuitState::HalfOpen);

        cb.check().unwrap();
        cb.record_failure();
        assert_eq!(cb.get_state(), CircuitState::Open);
    }

    #[test]
    fn test_circuit_breaker_reset() {
        let config = CircuitBreakerConfig::new(1);
        let cb = CircuitBreaker::new("test", config);

        cb.record_failure();
        assert_eq!(cb.get_state(), CircuitState::Open);

        cb.reset();
        assert_eq!(cb.get_state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_stats() {
        let config = CircuitBreakerConfig::new(5);
        let cb = CircuitBreaker::new("test", config);

        cb.record_success();
        cb.record_success();
        cb.record_failure();

        let stats = cb.get_stats();
        assert_eq!(stats.state, CircuitState::Closed);
        assert_eq!(stats.success_count, 2);
        assert_eq!(stats.failure_count, 1);
        assert_eq!(stats.failures_in_window, 1);
    }

    #[tokio::test]
    async fn test_circuit_breaker_call_success() {
        let cb = CircuitBreaker::new("test", CircuitBreakerConfig::default());

        let result = cb
            .call(|| async { Ok::<_, std::io::Error>("success") })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(cb.get_stats().success_count, 1);
    }

    #[tokio::test]
    async fn test_circuit_breaker_call_failure() {
        let cb = CircuitBreaker::new("test", CircuitBreakerConfig::default());

        let result: Result<(), CircuitBreakerError<std::io::Error>> = cb
            .call(|| async {
                Err(std::io::Error::new(std::io::ErrorKind::Other, "failed"))
            })
            .await;

        assert!(result.is_err());
        assert_eq!(cb.get_stats().failure_count, 1);
    }

    #[test]
    fn test_circuit_breaker_manager() {
        let manager = CircuitBreakerManager::default();

        let cb1 = manager.get_or_create("service1");
        let cb2 = manager.get_or_create("service2");
        let cb1_again = manager.get_or_create("service1");

        assert_eq!(cb1.name(), "service1");
        assert_eq!(cb2.name(), "service2");
        assert!(Arc::ptr_eq(&cb1, &cb1_again));
    }

    #[test]
    fn test_circuit_breaker_manager_custom_config() {
        let manager = CircuitBreakerManager::default();

        let config = CircuitBreakerConfig::new(10);
        let cb = manager.create_with_config("custom", config);

        // The circuit should be using custom config
        assert_eq!(cb.name(), "custom");
        assert_eq!(cb.get_state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_manager_get_all_stats() {
        let manager = CircuitBreakerManager::default();

        manager.get_or_create("a").record_success();
        manager.get_or_create("b").record_failure();

        let stats = manager.get_all_stats();
        assert_eq!(stats.len(), 2);
    }

    #[test]
    fn test_circuit_breaker_manager_reset_all() {
        let manager = CircuitBreakerManager::new(CircuitBreakerConfig::new(1));

        let cb = manager.get_or_create("test");
        cb.record_failure();
        assert_eq!(cb.get_state(), CircuitState::Open);

        manager.reset_all();
        assert_eq!(cb.get_state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_state_display() {
        assert_eq!(format!("{}", CircuitState::Closed), "closed");
        assert_eq!(format!("{}", CircuitState::Open), "open");
        assert_eq!(format!("{}", CircuitState::HalfOpen), "half-open");
    }
}
