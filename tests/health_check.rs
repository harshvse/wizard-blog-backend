use std::net::TcpListener;

use sqlx::{Connection, PgConnection};
use wizard_blog_backend::configuration::get_configuration;

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
    let server = wizard_blog_backend::startup::run(listener).expect("failed to create a server");

    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let addr = spawn_app();

    let configuration = get_configuration().expect("Failed to read config");
    let connection_string = configuration.database.connection_string();

    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to postgres.");

    let client = reqwest::Client::new();

    let body = "name=harsh%20verma&email=harshvse%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &addr))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&mut connection)
        .await
        .expect("failed to fetch saved subscription");

    assert_eq!(saved.email, "harshvse@gmail.com");
    assert_eq!(saved.name, "harsh verma");
}

#[tokio::test]
async fn subscribe_returns_400_for_invalid_form_data() {
    let addr = spawn_app();
    let client = reqwest::Client::new();

    let payloads = vec![
        ("", "both email and name are missing"),
        ("name=harsh%20verma", "email is missing"),
        ("email=harshvse%40gmail.com", "name is missing"),
    ];
    for (invalid_body, error_message) in payloads {
        let response = client
            .post(&format!("{}/subscriptions", &addr))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "the API did not return 400 when body was: {}.",
            error_message
        );
    }
}
