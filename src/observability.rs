//! Wires tracing (logs + spans) and OpenTelemetry (traces + metrics) per
//! ADR-0004. Exporters point at stdout until a real collector/backend is
//! deployed — this fixes the emission format and in-code API, not the backend.

use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};

pub fn init() {
    let tracer_provider = SdkTracerProvider::builder()
        .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
        .build();
    let meter_provider = SdkMeterProvider::builder()
        .with_periodic_exporter(opentelemetry_stdout::MetricExporter::default())
        .build();

    let tracer = tracer_provider.tracer("distolocal");
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"));

    Registry::default()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .with(otel_layer)
        .init();

    opentelemetry::global::set_meter_provider(meter_provider);
}
