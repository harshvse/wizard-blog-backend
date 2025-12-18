use std::net::TcpListener;

use wizard_blog_backend::configuration::get_configuration;
use wizard_blog_backend::startup::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration = get_configuration().expect("failed to read configuration.");

    let address = format!("127.0.0.1:{}", configuration.application_port);

    let listener: TcpListener = TcpListener::bind(address)?;

    println!(
        "starting server on port: {}",
        listener
            .local_addr()
            .expect("failed to get local addr")
            .port()
    );

    run(listener)?.await
}
