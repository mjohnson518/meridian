//! Business metrics for Prometheus scraping.
//!
//! Registered against the global Prometheus registry from `telemetry::prometheus_registry()`.
//!
//! Metrics:
//!   meridian_operations_total      — Counter  {type, status}
//!   meridian_reserve_ratio         — Gauge    {currency}
//!   meridian_attestation_age_secs  — Gauge    (seconds since last on-chain attestation)
//!   meridian_custody_balance       — Gauge    {asset}

use crate::telemetry::prometheus_registry;
use prometheus::{Gauge, GaugeVec, IntCounterVec, Opts};
use std::sync::OnceLock;

static OPERATIONS_TOTAL: OnceLock<IntCounterVec> = OnceLock::new();
static RESERVE_RATIO: OnceLock<GaugeVec> = OnceLock::new();
static ATTESTATION_AGE_SECS: OnceLock<Gauge> = OnceLock::new();
static CUSTODY_BALANCE: OnceLock<GaugeVec> = OnceLock::new();

/// Register all business metrics against the global Prometheus registry.
/// Safe to call multiple times — subsequent calls are no-ops.
pub fn init_metrics() {
    let registry = prometheus_registry();

    // meridian_operations_total{type="mint|burn", status="completed|failed|pending"}
    if OPERATIONS_TOTAL.get().is_none() {
        let counter = IntCounterVec::new(
            Opts::new(
                "meridian_operations_total",
                "Total mint/burn operations by type and status",
            ),
            &["type", "status"],
        )
        .expect("Failed to create operations counter");
        registry.register(Box::new(counter.clone())).ok();
        OPERATIONS_TOTAL.set(counter).ok();
    }

    // meridian_reserve_ratio{currency="EUR|GBP|..."}  — basis points (10000 = 100%)
    if RESERVE_RATIO.get().is_none() {
        let gauge = GaugeVec::new(
            Opts::new(
                "meridian_reserve_ratio",
                "Current reserve ratio in basis points per currency (10000 = 100%)",
            ),
            &["currency"],
        )
        .expect("Failed to create reserve ratio gauge");
        registry.register(Box::new(gauge.clone())).ok();
        RESERVE_RATIO.set(gauge).ok();
    }

    // meridian_attestation_age_secs — seconds since last on-chain reserve attestation
    if ATTESTATION_AGE_SECS.get().is_none() {
        let gauge = Gauge::new(
            "meridian_attestation_age_secs",
            "Seconds elapsed since the last on-chain reserve attestation",
        )
        .expect("Failed to create attestation age gauge");
        registry.register(Box::new(gauge.clone())).ok();
        ATTESTATION_AGE_SECS.set(gauge).ok();
    }

    // meridian_custody_balance{asset="EUR|GBP|..."}  — USD value of custody holdings
    if CUSTODY_BALANCE.get().is_none() {
        let gauge = GaugeVec::new(
            Opts::new(
                "meridian_custody_balance",
                "USD value of custody holdings per asset",
            ),
            &["asset"],
        )
        .expect("Failed to create custody balance gauge");
        registry.register(Box::new(gauge.clone())).ok();
        CUSTODY_BALANCE.set(gauge).ok();
    }
}

/// Increment the operations counter.
///
/// - `op_type`: `"mint"` or `"burn"`
/// - `status`:  `"completed"`, `"failed"`, or `"pending"`
pub fn record_operation(op_type: &str, status: &str) {
    if let Some(counter) = OPERATIONS_TOTAL.get() {
        counter.with_label_values(&[op_type, status]).inc();
    }
}

/// Set the reserve ratio gauge for a currency.
/// `ratio` is in basis points (10000 = 100%).
pub fn set_reserve_ratio(currency: &str, ratio_bps: f64) {
    if let Some(gauge) = RESERVE_RATIO.get() {
        gauge.with_label_values(&[currency]).set(ratio_bps);
    }
}

/// Set the seconds elapsed since the last on-chain reserve attestation.
pub fn set_attestation_age_secs(seconds: f64) {
    if let Some(gauge) = ATTESTATION_AGE_SECS.get() {
        gauge.set(seconds);
    }
}

/// Set the custody balance for an asset (USD value).
pub fn set_custody_balance(asset: &str, usd_value: f64) {
    if let Some(gauge) = CUSTODY_BALANCE.get() {
        gauge.with_label_values(&[asset]).set(usd_value);
    }
}
