use actix_web::{HttpRequest, HttpResponse, web};
use actix_web_flash_messages::FlashMessage;
use sqlx::PgPool;

use crate::{
    authentication::UserId,
    domain::SubscriberEmail,
    email_client::EmailClient,
    utils::{e500, see_other},
};

#[derive(serde::Deserialize)]
pub struct NewsletterFormData {
    title: String,
    html_content: String,
    text_content: String,
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[tracing::instrument(
    name = "Publish a new newsletter",
    skip(body, pool, email_client)
    fields(
        email_title = %body.title,
        email_text_content = %body.text_content,
        email_html_content = %body.html_content,
        user_id=tracing::field::Empty,
    )
)]
pub async fn publish_newsletter(
    body: web::Form<NewsletterFormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    user_id: web::ReqData<UserId>,
    request: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user_id.into_inner();
    tracing::Span::current().record("user_id", &tracing::field::display(&user_id.to_string()));

    if body.title.is_empty() {
        FlashMessage::error("Newsletter Publish Failed").send();
        return Ok(see_other("/admin/newsletters"));
    }

    let confirmed_subscribers = get_confirmed_subscribers(&pool).await.map_err(e500)?;

    for subscriber in confirmed_subscribers {
        match subscriber {
            Ok(subscriber) => {
                email_client
                    .send_email(
                        &subscriber.email,
                        &body.title,
                        &body.html_content,
                        &body.text_content,
                    )
                    .await
                    .map_err(e500)?;
            }
            Err(error) => {
                tracing::warn!(error.cause_chain  = ?error, "Skipping a confirmed subscriber. \
                    Their stored contact details are invalid");
            }
        }
    }
    FlashMessage::info("The newsletter issue has been published!").send();
    Ok(see_other("/admin/newsletters"))
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let confirmed_subscribers = sqlx::query!(
        r#"
            SELECT email FROM subscriptions
            WHERE status = 'confirmed'
        "#,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| match SubscriberEmail::parse(r.email) {
        Ok(email) => Ok(ConfirmedSubscriber { email }),
        Err(error) => Err(anyhow::anyhow!(error)),
    })
    .collect();
    Ok(confirmed_subscribers)
}
