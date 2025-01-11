use std::collections::HashMap;

use base64::Engine;
use opentelemetry::{trace::TracerProvider, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    trace::{Config, Sampler},
    Resource,
};
use tracing_subscriber::{filter::FilterFn, layer::SubscriberExt, util::SubscriberInitExt, Layer};

use crate::SERVICE_NAME;

pub fn init_telemetry(
    endpoint: &str,
    email: &str,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let auth = base64::engine::general_purpose::STANDARD.encode(format!("{}:{}", email, password));
    let mut headers = HashMap::new();
    headers.insert(
        "Authorization".to_string(),
        format!("Basic {}", auth).parse().unwrap(),
    );

    let provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint(endpoint)
                .with_headers(headers),
        )
        .with_trace_config(
            Config::default()
                .with_sampler(Sampler::AlwaysOn)
                .with_resource(Resource::new(vec![
                    KeyValue::new("service.name", env!("CARGO_PKG_NAME")),
                    KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                ])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    let tracer = provider.tracer(SERVICE_NAME);
    opentelemetry::global::set_tracer_provider(provider);

    let filter = FilterFn::new(|metadata| {
        let target = metadata.target();
        !target.starts_with("hyper")
            && !target.starts_with("tungstenite")
            && !target.starts_with("reqwest")
            && !target.starts_with("tungstenite")
            && !target.starts_with("tokio_tungstenite")
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
