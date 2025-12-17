use std::net::TcpListener;

use wizard_blog_backend::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let listener: TcpListener =
        TcpListener::bind("127.0.0.1:0").expect("failed to assign a port to the server");
    println!(
        "starting server on port: {}",
        listener
            .local_addr()
            .expect("failed to get local addr")
            .port()
    );
    run(listener)?.await
}
