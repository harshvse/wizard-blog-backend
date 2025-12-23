use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use wizard_blog_backend::{
    configuration::{DatabaseSettings, get_configuration},
    telemetry::{get_subscriber, init_subscriber},
};

pub struct TestApp {
    address: String,
    db_pool: PgPool,
}

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

async fn spawn_app() -> TestApp {
    // Called once and skipped for rest of the calls
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to assign a port to the server");
    let port = listener
        .local_addr()
        .expect("failed to get local addr")
        .port();

    let mut configuration = get_configuration().expect("failed to load configuration");

    configuration.database.database_name = Uuid::new_v4().to_string();

    let db_pool = configure_database(&configuration.database).await;

    let server = wizard_blog_backend::startup::run(listener, db_pool.clone())
        .expect("failed to create a server");

    let _ = tokio::spawn(server);

    let address = format!("http://127.0.0.1:{}", port);
    TestApp { address, db_pool }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("failed to connect to db");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("failed to create new database");

    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("failed to connect to postgres");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect(" failed to migrate the db");

    connection_pool
}

#[tokio::test]
async fn health_check_works() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health", test_app.address))
        .send()
        .await
        .expect("failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let test_app = spawn_app().await;

    let client = reqwest::Client::new();

    let body = "name=harsh%20verma&email=harshvse%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &test_app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&test_app.db_pool)
        .await
        .expect("failed to fetch saved subscription");

    assert_eq!(saved.email, "harshvse@gmail.com");
    assert_eq!(saved.name, "harsh verma");
}

#[tokio::test]
async fn subscribe_returns_400_for_invalid_form_data() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let payloads = vec![
        ("", "both email and name are missing"),
        ("name=harsh%20verma", "email is missing"),
        ("email=harshvse%40gmail.com", "name is missing"),
    ];
    for (invalid_body, error_message) in payloads {
        let response = client
            .post(&format!("{}/subscriptions", &test_app.address))
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

#[tokio::test]
async fn subscribe_returns_400_when_fields_are_present_but_invalid() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=&email=harshvse%40gmail.com", "empty name"),
        ("name=harsh%20verma&email=", "empty email"),
        ("name=harsh%20verma&email=def-not-an-email", "invalid email"),
    ];

    for (body, description) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("failed to execute request");
        assert_eq!(
            400,
            response.status().as_u16(),
            "the API did not return 400 when body was:{}.",
            description
        );
    }
}
