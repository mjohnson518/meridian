//! Application state shared across all handlers

use meridian_oracle::ChainlinkOracle;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// CRIT-002: Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, requests flow normally
    Closed,
    /// Circuit is open, requests are immediately rejected
    Open,
    /// Circuit is testing if the service has recovered
    HalfOpen,
}

/// CRIT-002: Circuit breaker for oracle calls
///
/// Prevents cascading failures by fast-failing when the oracle is unavailable.
/// Uses atomic operations for thread-safe state management.
pub struct CircuitBreaker {
    /// Number of consecutive failures
    failure_count: AtomicU32,
    /// Timestamp (epoch ms) when circuit opened
    opened_at: AtomicU64,
    /// Number of consecutive failures to trip the circuit
    failure_threshold: u32,
    /// How long (ms) to wait before trying half-open
    reset_timeout_ms: u64,
    /// Number of successes needed in half-open to close
    success_threshold: u32,
    /// Consecutive successes in half-open state
    half_open_successes: AtomicU32,
}

impl CircuitBreaker {
    /// Creates a new circuit breaker with default settings
    /// - Opens after 5 consecutive failures
    /// - Waits 30 seconds before testing half-open
    /// - Requires 2 successes in half-open to close
    pub fn new() -> Self {
        Self {
            failure_count: AtomicU32::new(0),
            opened_at: AtomicU64::new(0),
            failure_threshold: 5,
            reset_timeout_ms: 30_000, // 30 seconds
            success_threshold: 2,
            half_open_successes: AtomicU32::new(0),
        }
    }

    /// Get current timestamp in milliseconds
    fn now_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    /// Get the current circuit state
    pub fn state(&self) -> CircuitState {
        let failures = self.failure_count.load(Ordering::SeqCst);
        let opened_at = self.opened_at.load(Ordering::SeqCst);

        if failures < self.failure_threshold {
            return CircuitState::Closed;
        }

        // Circuit has tripped - check if timeout has elapsed
        let elapsed = Self::now_ms().saturating_sub(opened_at);
        if elapsed >= self.reset_timeout_ms {
            CircuitState::HalfOpen
        } else {
            CircuitState::Open
        }
    }

    /// Check if a request should be allowed
    /// Returns true if the request can proceed, false if circuit is open
    pub fn allow_request(&self) -> bool {
        match self.state() {
            CircuitState::Closed => true,
            CircuitState::Open => false,
            CircuitState::HalfOpen => true, // Allow test requests
        }
    }

    /// Record a successful request
    pub fn record_success(&self) {
        let state = self.state();
        match state {
            CircuitState::Closed => {
                // Already healthy, nothing to do
            }
            CircuitState::HalfOpen => {
                let successes = self.half_open_successes.fetch_add(1, Ordering::SeqCst) + 1;
                if successes >= self.success_threshold {
                    // Reset the circuit
                    self.failure_count.store(0, Ordering::SeqCst);
                    self.opened_at.store(0, Ordering::SeqCst);
                    self.half_open_successes.store(0, Ordering::SeqCst);
                    tracing::info!("Circuit breaker CLOSED after {} successes in half-open", successes);
                }
            }
            CircuitState::Open => {
                // Shouldn't happen, but record anyway
                self.half_open_successes.fetch_add(1, Ordering::SeqCst);
            }
        }
    }

    /// Record a failed request
    pub fn record_failure(&self) {
        let failures = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;

        // Reset half-open successes on any failure
        self.half_open_successes.store(0, Ordering::SeqCst);

        if failures >= self.failure_threshold {
            let was_open = self.opened_at.load(Ordering::SeqCst) > 0;
            if !was_open {
                self.opened_at.store(Self::now_ms(), Ordering::SeqCst);
                tracing::warn!(
                    failures = failures,
                    threshold = self.failure_threshold,
                    "Circuit breaker OPENED after {} consecutive failures",
                    failures
                );
            }
        }
    }

    /// Get metrics for monitoring
    pub fn metrics(&self) -> CircuitBreakerMetrics {
        CircuitBreakerMetrics {
            state: self.state(),
            failure_count: self.failure_count.load(Ordering::SeqCst),
            opened_at: self.opened_at.load(Ordering::SeqCst),
        }
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

/// Circuit breaker metrics for monitoring
#[derive(Debug, Clone)]
pub struct CircuitBreakerMetrics {
    pub state: CircuitState,
    pub failure_count: u32,
    pub opened_at: u64,
}

/// Shared application state
pub struct AppState {
    /// Database connection pool
    pub db_pool: Arc<PgPool>,
    /// Chainlink oracle client (optional, requires RPC URL)
    pub oracle: Arc<RwLock<Option<ChainlinkOracle>>>,
    /// CRIT-002: Circuit breaker for oracle calls
    pub oracle_circuit_breaker: CircuitBreaker,
}

impl AppState {
    /// Creates new application state with database pool
    pub async fn new(db_pool: PgPool) -> Self {
        // Try to initialize oracle if RPC URL is provided
        let oracle = if let Ok(rpc_url) = std::env::var("ETHEREUM_RPC_URL") {
            tracing::info!("Initializing Chainlink oracle with RPC URL");
            match ChainlinkOracle::new(&rpc_url, Decimal::new(10, 0)).await {
                Ok(oracle) => {
                    tracing::info!("Chainlink oracle initialized");
                    Some(oracle)
                }
                Err(e) => {
                    tracing::warn!("Failed to initialize oracle: {}", e);
                    None
                }
            }
        } else {
            tracing::info!("No ETHEREUM_RPC_URL provided, oracle features disabled");
            None
        };

        Self {
            db_pool: Arc::new(db_pool),
            oracle: Arc::new(RwLock::new(oracle)),
            oracle_circuit_breaker: CircuitBreaker::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_starts_closed() {
        let cb = CircuitBreaker::new();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.allow_request());
    }

    #[test]
    fn test_circuit_breaker_opens_after_threshold() {
        let cb = CircuitBreaker::new();

        // Record 5 failures (threshold is 5)
        for _ in 0..5 {
            cb.record_failure();
        }

        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.allow_request());
    }

    #[test]
    fn test_circuit_breaker_success_resets_in_half_open() {
        let cb = CircuitBreaker {
            failure_count: std::sync::atomic::AtomicU32::new(5),
            opened_at: std::sync::atomic::AtomicU64::new(1), // Long ago - will be half-open
            failure_threshold: 5,
            reset_timeout_ms: 0, // Instant timeout for test
            success_threshold: 2,
            half_open_successes: std::sync::atomic::AtomicU32::new(0),
        };

        // Should be half-open since timeout elapsed
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        // Record 2 successes
        cb.record_success();
        cb.record_success();

        // Should be closed now
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_failure_in_half_open_resets_successes() {
        let cb = CircuitBreaker {
            failure_count: std::sync::atomic::AtomicU32::new(5),
            opened_at: std::sync::atomic::AtomicU64::new(1),
            failure_threshold: 5,
            reset_timeout_ms: 0,
            success_threshold: 2,
            half_open_successes: std::sync::atomic::AtomicU32::new(1),
        };

        // Record 1 success
        assert_eq!(cb.half_open_successes.load(std::sync::atomic::Ordering::SeqCst), 1);

        // Record failure
        cb.record_failure();

        // Half-open successes should be reset
        assert_eq!(cb.half_open_successes.load(std::sync::atomic::Ordering::SeqCst), 0);
    }
}
