//! OpenTelemetry and Prometheus telemetry configuration
//!
//! CRIT-003: Distributed tracing with OpenTelemetry
//! CRIT-004: Prometheus metrics instrumentation

use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{runtime, trace::Sampler, Resource};
use prometheus::Registry;
use std::sync::OnceLock;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Global Prometheus registry for metrics
static PROMETHEUS_REGISTRY: OnceLock<Registry> = OnceLock::new();

/// Get the global Prometheus registry
pub fn prometheus_registry() -> &'static Registry {
    PROMETHEUS_REGISTRY.get_or_init(Registry::new)
}

/// Telemetry configuration
#[derive(Clone, Debug)]
pub struct TelemetryConfig {
    /// Service name for traces
    pub service_name: String,
    /// OTLP endpoint for trace export (e.g., http://localhost:4317)
    pub otlp_endpoint: Option<String>,
    /// Sampling rate (0.0 to 1.0)
    pub sampling_rate: f64,
    /// Whether to enable JSON logging format
    pub json_logs: bool,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            service_name: "meridian-api".to_string(),
            otlp_endpoint: None,
            sampling_rate: 1.0, // Sample all traces by default
            json_logs: false,
        }
    }
}

impl TelemetryConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        let service_name =
            std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "meridian-api".to_string());

        let otlp_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok();

        let sampling_rate = std::env::var("OTEL_TRACES_SAMPLER_ARG")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1.0);

        let json_logs = std::env::var("LOG_FORMAT")
            .map(|f| f.to_lowercase() == "json")
            .unwrap_or(false);

        Self {
            service_name,
            otlp_endpoint,
            sampling_rate,
            json_logs,
        }
    }
}

/// Initialize telemetry (tracing + metrics)
///
/// This sets up:
/// - Tracing subscriber with OpenTelemetry integration
/// - OTLP exporter for distributed tracing (if configured)
/// - Prometheus metrics registry
///
/// # Panics
/// Panics if telemetry initialization fails (considered critical)
pub fn init_telemetry(config: TelemetryConfig) {
    // Initialize Prometheus registry
    let _ = PROMETHEUS_REGISTRY.set(Registry::new());

    // Create EnvFilter for log levels
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,meridian_api=debug"));

    // Build the tracing subscriber based on configuration
    match (&config.otlp_endpoint, config.json_logs) {
        // OTLP enabled + JSON logs
        (Some(endpoint), true) => {
            let tracer_provider =
                init_tracer_provider(&config.service_name, endpoint, config.sampling_rate);
            let tracer = tracer_provider.tracer("meridian-api");
            let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer().json())
                .with(otel_layer)
                .init();

            tracing::info!(
                service_name = %config.service_name,
                otlp_endpoint = %endpoint,
                sampling_rate = %config.sampling_rate,
                json_logs = true,
                "OpenTelemetry tracing initialized with OTLP export (JSON format)"
            );
        }
        // OTLP enabled + standard logs
        (Some(endpoint), false) => {
            let tracer_provider =
                init_tracer_provider(&config.service_name, endpoint, config.sampling_rate);
            let tracer = tracer_provider.tracer("meridian-api");
            let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer())
                .with(otel_layer)
                .init();

            tracing::info!(
                service_name = %config.service_name,
                otlp_endpoint = %endpoint,
                sampling_rate = %config.sampling_rate,
                "OpenTelemetry tracing initialized with OTLP export"
            );
        }
        // No OTLP + JSON logs
        (None, true) => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer().json())
                .init();

            tracing::info!(
                service_name = %config.service_name,
                json_logs = true,
                "Telemetry initialized (OTLP disabled, JSON format)"
            );
        }
        // No OTLP + standard logs
        (None, false) => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer())
                .init();

            tracing::info!(
                service_name = %config.service_name,
                "Telemetry initialized (OTLP disabled - set OTEL_EXPORTER_OTLP_ENDPOINT to enable)"
            );
        }
    }
}

/// Initialize the OpenTelemetry tracer provider
fn init_tracer_provider(
    service_name: &str,
    endpoint: &str,
    sampling_rate: f64,
) -> opentelemetry_sdk::trace::TracerProvider {
    let sampler = if sampling_rate >= 1.0 {
        Sampler::AlwaysOn
    } else if sampling_rate <= 0.0 {
        Sampler::AlwaysOff
    } else {
        Sampler::TraceIdRatioBased(sampling_rate)
    };

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()
        .expect("Failed to create OTLP exporter");

    opentelemetry_sdk::trace::TracerProvider::builder()
        .with_batch_exporter(exporter, runtime::Tokio)
        .with_sampler(sampler)
        .with_resource(Resource::new(vec![
            opentelemetry::KeyValue::new("service.name", service_name.to_string()),
            opentelemetry::KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
        ]))
        .build()
}

/// Shutdown telemetry gracefully
/// Call this before application exit to flush pending traces
pub fn shutdown_telemetry() {
    opentelemetry::global::shutdown_tracer_provider();
    tracing::info!("Telemetry shutdown complete");
}

/// Generate Prometheus metrics output
pub fn prometheus_metrics() -> String {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let metric_families = prometheus_registry().gather();
    let mut buffer = Vec::new();
    encoder
        .encode(&metric_families, &mut buffer)
        .unwrap_or_default();
    String::from_utf8(buffer).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TelemetryConfig::default();
        assert_eq!(config.service_name, "meridian-api");
        assert!(config.otlp_endpoint.is_none());
        assert_eq!(config.sampling_rate, 1.0);
        assert!(!config.json_logs);
    }

    #[test]
    fn test_prometheus_registry() {
        let registry = prometheus_registry();
        // Should return the same registry on subsequent calls
        let registry2 = prometheus_registry();
        assert!(std::ptr::eq(registry, registry2));
    }
}
