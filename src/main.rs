use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use wizard_blog_backend::{
    configuration::get_configuration,
    email_client::EmailClient,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("wizard-blog-backend".into(), "info".into(), std::io::stdout);

    init_subscriber(subscriber);

    let configuration = get_configuration().expect("failed to read configuration.");

    let address = format!(
        "{}:{}",
        configuration.application_host, configuration.application_port
    );
    let timeout = configuration.email_client.timeout();

    let listener: TcpListener = TcpListener::bind(address)?;

    let connection_pool = PgPoolOptions::new().connect_lazy_with(configuration.database.with_db());

    let sender_email = configuration
        .email_client
        .sender()
        .expect("failed to get sender email");

    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.auth_token,
        timeout,
    );

    println!(
        "starting server on port: {}",
        listener
            .local_addr()
            .expect("failed to get local addr")
            .port()
    );

    run(listener, connection_pool, email_client)?.await?;
    Ok(())
}
