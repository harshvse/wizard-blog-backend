use actix_web::{HttpResponse, Responder, web};
use chrono::Utc;
use rand::{Rng, distr::Alphanumeric, rng};
use sqlx::{Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::EmailClient,
    startup::ApplicationBaseUrl,
};

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let email = SubscriberEmail::parse(value.email)?;
        let name = SubscriberName::parse(value.name)?;

        Ok(Self { email, name })
    }
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, db_pool, email_client, base_url),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    db_pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
) -> impl Responder {
    let new_subscriber: NewSubscriber = match form.0.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    let mut transaction = match db_pool.begin().await {
        Ok(transaction) => transaction,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let subscriber_id = match insert_subscriber(&mut transaction, &new_subscriber).await {
        Ok(subscriber_id) => subscriber_id,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    let token = generate_subscription_token();

    // store the token in database
    match store_token(&mut transaction, subscriber_id, &token).await {
        Ok(_) => (),
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    if transaction.commit().await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    match send_confirmation_email(&email_client, new_subscriber, &base_url.0, &token).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    };

    HttpResponse::Ok().finish()
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, transaction)
)]
pub async fn insert_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    new_subscriber: &NewSubscriber,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    let query = sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at, status)
            VALUES ($1, $2, $3, $4, 'pending_confirmation')
        "#,
        subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    );
    transaction.execute(query).await.map_err(|e| {
        tracing::error!("failed to execute query: {:?}", e);
        e
    })?;
    Ok(subscriber_id)
}

#[tracing::instrument(
    name = "Send confirmation email after saving the user in database",
    skip(email_client, new_subscriber, base_url, token)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
    token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, token
    );

    let subject = "Welcome";
    let html_body = &format!(
        "Welcome to our newsletter!<br />\
            Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );
    let plain_body = &format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );
    email_client
        .send_email(new_subscriber.email, subject, &html_body, &plain_body)
        .await
}

fn generate_subscription_token() -> String {
    let mut rng = rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric) as char)
        .take(50)
        .collect()
}

#[tracing::instrument(
    name = "Store confirmation token in database",
    skip(transaction, subscriber_id, token)
)]
async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    token: &str,
) -> Result<(), sqlx::Error> {
    let query = sqlx::query!(
        r#"
            INSERT INTO subscription_tokens(subscription_token, subscriber_id)
            VALUES ($1, $2)
        "#,
        token,
        subscriber_id,
    );
    transaction.execute(query).await.map_err(|e| {
        tracing::error!("failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
