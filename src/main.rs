use sqlx::PgPool;
use std::net::TcpListener;
use wizard_blog_backend::configuration::get_configuration;
use wizard_blog_backend::startup::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
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
