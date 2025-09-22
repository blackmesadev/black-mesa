use std::collections::HashMap;

use base64::Engine;
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::{Protocol, WithExportConfig, WithHttpConfig};
use opentelemetry_sdk::Resource;
use tracing_subscriber::{filter::FilterFn, layer::SubscriberExt, util::SubscriberInitExt, Layer};

use crate::SERVICE_NAME;

pub fn init_telemetry(
    endpoint: &str,
    auth: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut headers = HashMap::with_capacity(1);
    if let Some(auth) = auth {
        headers.insert(
            String::from("Authorization"),
            format!(
                "Basic {}",
                base64::engine::general_purpose::STANDARD.encode(auth)
            ),
        );
    }

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_http_client(reqwest::Client::new())
        .with_protocol(Protocol::HttpBinary)
        .with_headers(headers)
        .with_endpoint(endpoint)
        .build()?;

    let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_resource(Resource::builder()
            .with_attributes(vec![opentelemetry::KeyValue::new(
                "service.name",
                SERVICE_NAME,
            )])
            .build())
        .with_simple_exporter(exporter)
        .build();

    let tracer = provider.tracer(SERVICE_NAME);
    opentelemetry::global::set_tracer_provider(provider);

    const FILTERED_TARGETS: &[&str] = &["hyper", "tungstenite", "reqwest", "tokio_tungstenite"];

    let filter = FilterFn::new(|metadata| {
        let target = metadata.target();
        !FILTERED_TARGETS
            .iter()
            .any(|&prefix| target.starts_with(prefix))
    });

    let telemetry = tracing_opentelemetry::layer()
        .with_tracer(tracer)
        .with_filter(filter.clone());

    let fmt = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .with_filter(filter);

    tracing_subscriber::registry()
        .with(telemetry)
        .with(fmt)
        .init();

    Ok(())
}
