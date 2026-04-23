use wiremock::{
    Mock, ResponseTemplate,
    matchers::{any, method, path},
};

use crate::helpers::{ConfirmationLinks, TestApp, assert_is_redirect_to, spawn_app};

#[tokio::test]
async fn newsletter_are_not_delivered_to_unconfirmed_subscribers() {
    let app = spawn_app().await;

    // Login
    app.post_login(&serde_json::json!({
    "username": &app.test_user.username,
    "password": &app.test_user.password
    }))
    .await;

    create_unconfirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title":"newsletter title",
            "text_content": "Newsletter body as plain text",
            "html_content": "<p>Newsletter body as Html</p>"
    });

    let response = app.post_newsletter(newsletter_request_body).await;

    assert_is_redirect_to(&response, "/admin/newsletters");
}

#[tokio::test]
async fn newsletter_are_delivered_to_confirmed_subscribers() {
    let app = spawn_app().await;

    // Login
    app.post_login(&serde_json::json!({
    "username": &app.test_user.username,
    "password": &app.test_user.password
    }))
    .await;

    create_confirmed_subscriber(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
            "title":"newsletter title",
            "text_content": "Newsletter body as plain text",
            "html_content": "<p>Newsletter body as Html</p>",
    });

    let response = app.post_newsletter(newsletter_request_body).await;

    assert_is_redirect_to(&response, "/admin/newsletters");
}

#[tokio::test]
async fn newsletters_returns_400_for_invalid_data() {
    // Arrange
    let app = spawn_app().await;

    // Login
    app.post_login(&serde_json::json!({
    "username": &app.test_user.username,
    "password": &app.test_user.password
    }))
    .await;

    let test_cases = vec![
        (
            serde_json::json!({
            "text_content": "Newsletter body as plain text",
            "html_content": "<p>Newsletter body as HTML</p>",
            }),
            "missing title",
        ),
        (
            serde_json::json!({"title": "Newsletter!"}),
            "missing content",
        ),
    ];
    for (invalid_body, error_message) in test_cases {
        let response = app.post_newsletter(invalid_body).await;
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn requests_missing_authorization_are_rejected() {
    let app = spawn_app().await;

    let newsletter_request_body = serde_json::json!({
            "title":"newsletter title",
            "text_content": "Newsletter body as plain text",
            "html_content": "<p>Newsletter body as Html</p>",
    });

    let response = app.post_newsletter(newsletter_request_body).await;

    assert_is_redirect_to(&response, "/login");
}

async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
    let body = "name=Harsh%20Verma&email=harshvse%40gmail.com";
    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;
    app.post_subscriptions(body.into())
        .await
        .error_for_status()
        .unwrap();

    let email_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();

    app.get_confirmation_links(&email_request)
}

async fn create_confirmed_subscriber(app: &TestApp) {
    let confirmation_links = create_unconfirmed_subscriber(app).await;

    reqwest::get(confirmation_links.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}
