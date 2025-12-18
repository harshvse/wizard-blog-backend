use sqlx::PgPool;
use std::net::TcpListener;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt};
use wizard_blog_backend::{configuration::get_configuration, startup::run};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // redirect all logs events to tracing
    LogTracer::init().expect("failed to initialize log tracer");
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let formatting_layer =
        BunyanFormattingLayer::new("wizard-blog-backend".into(), std::io::stdout);

    let subscriber = Registry::default()
        .with(env_filter)
        .with(formatting_layer)
        .with(JsonStorageLayer);

    set_global_default(subscriber).expect("failed to set subscriber");

    let configuration = get_configuration().expect("failed to read configuration.");

    let address = format!("127.0.0.1:{}", configuration.application_port);

    let listener: TcpListener = TcpListener::bind(address)?;

    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("failed to connect to db");

    println!(
        "starting server on port: {}",
        listener
            .local_addr()
            .expect("failed to get local addr")
            .port()
    );

    run(listener, connection_pool)?.await
}
