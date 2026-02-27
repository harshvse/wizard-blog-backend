use crate::helpers::{assert_is_redirect_to, spawn_app};

#[tokio::test]
async fn an_error_flash_message_is_set_on_failure() {
    let app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": "random-username",
        "password":"random-password",
    });

    let response = app.post_login(&login_body).await;

    assert_is_redirect_to(&response, "/login");

    // Act
    let login_page = app.get_login_form().await;
    assert!(login_page.contains(r#"<p><i>Authentication failed</i></p>"#));

    // Cookie should have expired
    let login_page = app.get_login_form().await;
    assert!(!login_page.contains(r#"<p><i>Authentication failed</i></p>"#));
}
