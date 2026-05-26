//! Observability setup.
//!
//! BACKEND.md §1.9 + §15:
//! - JSON `tracing` to stdout — **always installed.**
//! - OTLP exporter via `tracing-opentelemetry` — **installed only if
//!   `OTEL_EXPORTER_OTLP_ENDPOINT` is set.** Local dev without a collector
//!   simply runs stdout-only.
//! - Prometheus listener on a separate port (`/metrics`). Hard label
//!   cardinality discipline lives at the call site, not here.

use common::config::Config;
use metrics_exporter_prometheus::PrometheusBuilder;
use std::net::SocketAddr;
use tracing_subscriber::{prelude::*, EnvFilter};

#[derive(Debug, thiserror::Error)]
pub enum ObsError {
    #[error("tracing init: {0}")]
    Tracing(String),

    #[error("metrics init: {0}")]
    Metrics(String),

    #[error("otlp init: {0}")]
    Otlp(String),
}

/// Initialize tracing + metrics for a binary. Call once at startup, before
/// any other code emits spans.
///
/// `service_name` is the OTEL service.name attribute (e.g. `"tradeflex-api"`).
/// If `config.otel_service_name` is set it overrides this argument.
pub fn init(config: &Config, service_name: &'static str) -> Result<ObservabilityGuard, ObsError> {
    let filter = EnvFilter::try_new(
        config
            .rust_log
            .as_deref()
            .unwrap_or("info,sqlx=warn,hyper=warn,h2=warn"),
    )
    .map_err(|e| ObsError::Tracing(e.to_string()))?;

    let json_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_current_span(true)
        .with_span_list(false)
        .with_target(true);

    let registry = tracing_subscriber::registry().with(filter).with(json_layer);

    let otel_guard = if let Some(endpoint) = config.otel_exporter_otlp_endpoint.as_deref() {
        let name = config
            .otel_service_name
            .clone()
            .unwrap_or_else(|| service_name.to_string());
        let tracer = build_otlp_tracer(endpoint, &name)?;
        let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
        registry
            .with(otel_layer)
            .try_init()
            .map_err(|e| ObsError::Tracing(e.to_string()))?;
        tracing::info!(endpoint = %endpoint, service = %name, "OTLP tracing enabled");
        Some(OtelGuard)
    } else {
        registry
            .try_init()
            .map_err(|e| ObsError::Tracing(e.to_string()))?;
        tracing::info!(
            "OTEL_EXPORTER_OTLP_ENDPOINT not set; OTLP tracing disabled (stdout JSON still on)"
        );
        None
    };

    let metrics_handle = install_prometheus(config.metrics_bind)?;

    Ok(ObservabilityGuard {
        _otel: otel_guard,
        _metrics: metrics_handle,
    })
}

fn build_otlp_tracer(
    endpoint: &str,
    service_name: &str,
) -> Result<opentelemetry_sdk::trace::Tracer, ObsError> {
    use opentelemetry::{trace::TracerProvider as _, KeyValue};
    use opentelemetry_otlp::WithExportConfig;
    use opentelemetry_sdk::{runtime, trace, Resource};

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()
        .map_err(|e| ObsError::Otlp(e.to_string()))?;

    let provider = trace::TracerProvider::builder()
        .with_batch_exporter(exporter, runtime::Tokio)
        .with_resource(Resource::new(vec![KeyValue::new(
            "service.name",
            service_name.to_string(),
        )]))
        .build();

    let tracer = provider.tracer(service_name.to_string());
    // Install as the global provider so downstream OTEL crates (e.g.
    // `opentelemetry::global::tracer`) see it.
    opentelemetry::global::set_tracer_provider(provider);
    Ok(tracer)
}

fn install_prometheus(addr: SocketAddr) -> Result<MetricsGuard, ObsError> {
    PrometheusBuilder::new()
        .with_http_listener(addr)
        .install()
        .map_err(|e| ObsError::Metrics(e.to_string()))?;
    tracing::info!(%addr, "Prometheus /metrics listener installed");
    Ok(MetricsGuard)
}

/// RAII guard: drop at process shutdown to flush OTLP exporters cleanly.
pub struct ObservabilityGuard {
    _otel: Option<OtelGuard>,
    _metrics: MetricsGuard,
}

pub struct OtelGuard;

impl Drop for OtelGuard {
    fn drop(&mut self) {
        opentelemetry::global::shutdown_tracer_provider();
    }
}

pub struct MetricsGuard;
