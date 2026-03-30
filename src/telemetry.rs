use opentelemetry::{
    global, propagation::TextMapCompositePropagator, trace::TracerProvider as _, KeyValue,
};
use opentelemetry_otlp::{SpanExporter, WithTonicConfig};
use opentelemetry_sdk::{
    propagation::{BaggagePropagator, TraceContextPropagator},
    trace::{BatchConfigBuilder, BatchSpanProcessor, TracerProvider as SdkTracerProvider},
    Resource,
};
use tonic::metadata::MetadataMap;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialise tracing with an OTLP exporter pointed at Tempo (or any
/// OTLP-compatible backend).  Returns the provider so callers can drive a
/// clean `shutdown()` on exit.
pub fn init(
    service_name: &'static str,
    otlp_endpoint: &str,
    otlp_auth: Option<&str>,
    otlp_organization: Option<&str>,
) -> SdkTracerProvider {
    let mut metadata = MetadataMap::new();
    if let Some(auth) = otlp_auth {
        metadata.insert(
            "authorization",
            auth.parse().expect("invalid auth header value"),
        );
    }
    if let Some(org) = otlp_organization {
        metadata.insert(
            "organization",
            org.parse().expect("invalid organization header value"),
        );
    }

    let channel = tonic::transport::Channel::from_shared(otlp_endpoint.to_string())
        .expect("invalid OTLP endpoint URI")
        .connect_lazy();

    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_channel(channel)
        .with_metadata(metadata)
        .build()
        .expect("Failed to build OTLP span exporter");

    let resource = Resource::new([KeyValue::new("service.name", service_name)]);

    let processor = BatchSpanProcessor::builder(exporter, opentelemetry_sdk::runtime::Tokio)
        .with_batch_config(
            BatchConfigBuilder::default()
                .with_scheduled_delay(std::time::Duration::from_secs(1))
                .build(),
        )
        .build();

    let provider = SdkTracerProvider::builder()
        .with_span_processor(processor)
        .with_resource(resource)
        .build();

    let tracer = provider.tracer(service_name);

    global::set_text_map_propagator(TextMapCompositePropagator::new(vec![
        Box::new(TraceContextPropagator::new()),
        Box::new(BaggagePropagator::new()),
    ]));

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,h2=off,hyper=off,tower=off")),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(OpenTelemetryLayer::new(tracer))
        .init();

    provider
}
