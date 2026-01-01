use opentelemetry::KeyValue;
use opentelemetry_sdk::{trace as sdktrace, Resource};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_telemetry() {
    // 1. Forest Layer (Console UI)
    let forest_layer = tracing_forest::ForestLayer::default();

    // 2. OTel Layer (Background Data)
    // Fix: We use the concrete type 'sdktrace::Tracer' here
    let otel_layer = init_tracer().map(|tracer| tracing_opentelemetry::layer().with_tracer(tracer));

    // 3. Filter Layer
    let filter_layer =
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());

    // 4. Register All Layers
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(forest_layer)
        .with(otel_layer)
        .init();
}

// Fix: Return 'sdktrace::Tracer' (Concrete Struct) instead of 'opentelemetry::trace::Tracer' (Trait)
fn init_tracer() -> Option<sdktrace::Tracer> {
    let resource = Resource::new(vec![KeyValue::new("service.name", env!("CARGO_PKG_NAME"))]);

    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .with_trace_config(sdktrace::config().with_resource(resource))
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .ok()
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}
