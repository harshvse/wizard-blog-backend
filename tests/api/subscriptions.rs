use crate::helpers::spawn_app;

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let test_app = spawn_app().await;

    let body = "name=harsh%20verma&email=harshvse%40gmail.com";
    let response = test_app.post_subscriptions(body.into()).await;

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
    let payloads = vec![
        ("", "both email and name are missing"),
        ("name=harsh%20verma", "email is missing"),
        ("email=harshvse%40gmail.com", "name is missing"),
    ];
    for (invalid_body, error_message) in payloads {
        let response = test_app.post_subscriptions(invalid_body.into()).await;

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
    let test_cases = vec![
        ("name=&email=harshvse%40gmail.com", "empty name"),
        ("name=harsh%20verma&email=", "empty email"),
        ("name=harsh%20verma&email=def-not-an-email", "invalid email"),
    ];
    for (body, description) in test_cases {
        let response = app.post_subscriptions(body.into()).await;
        assert_eq!(
            400,
            response.status().as_u16(),
            "the API did not return 400 when body was:{}.",
            description
        );
    }
}
