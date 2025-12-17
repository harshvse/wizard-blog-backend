use std::net::TcpListener;

#[tokio::test]
async fn health_check_works() {
    let addr = spawn_app();
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health", addr))
        .send()
        .await
        .expect("failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to assign a port to the server");
    let port = listener
        .local_addr()
        .expect("failed to get local addr")
        .port();
    let server = wizard_blog_backend::run(listener).expect("failed to create a server");

    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
