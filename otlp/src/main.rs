// use opentelemetry::global;
// use opentelemetry::trace::Tracer;
// use opentelemetry_otlp::WithExportConfig;

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
//     // Initialize OTLP exporter using gRPC (Tonic)
//     tracing_subscriber::fmt().with_env_filter("debug").init();
//     let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
//         .with_tonic()
//         .with_endpoint("http://host.docker.internal:4317")
//         .build()?;

//     // Create a tracer provider with the exporter
//     let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
//         .with_batch_exporter(otlp_exporter)
//         .build();

//     // Set it as the global provider
//     global::set_tracer_provider(tracer_provider.clone());

//     // Get a tracer and create spans
//     let tracer = global::tracer("my_tracer");
//     tracer.in_span("doing_work", |_cx| {
//         // Your application logic here...
//         println!("sending test span");
//     });
//     tracer_provider.shutdown()?;
//     Ok(())
// }
use opentelemetry::global;
use opentelemetry_otlp::WithExportConfig;
use tracing::{Level, span};
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    // OTLP exporter
    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint("http://host.docker.internal:4317")
        .build()?;

    // Tracer provider
    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(otlp_exporter)
        .build();

    global::set_tracer_provider(tracer_provider.clone());

    // Get a tracer and create spans
    let tracer = global::tracer("my_tracer");

    // tracing-opentelemetry layer
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // subscriber
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("debug"))
        .with(tracing_subscriber::fmt::layer())
        .with(otel_layer)
        .init();

    // ===== ここが tracing span =====
    {
        let span = span!(Level::INFO, "doing_work_da");
        let _enter = span.enter();

        println!("sending test span");
        tracing::info!("This is an info log with OTLP tracing");
        tracing::debug!("This is a debug log with OTLP tracing");
    }
    // flush
    tracer_provider.shutdown()?;

    Ok(())
}
