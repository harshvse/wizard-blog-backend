use crate::routes::helper::error_chain_fmt;
use actix_web::{HttpResponse, ResponseError, http::StatusCode, web};
use anyhow::Context;
use serde::Deserialize;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct Parameters {
    subscription_token: String,
}

#[derive(thiserror::Error)]
pub enum SubscriptionConfirmError {
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl From<String> for SubscriptionConfirmError {
    fn from(e: String) -> Self {
        Self::ValidationError(e)
    }
}
impl std::fmt::Debug for SubscriptionConfirmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for SubscriptionConfirmError {
    fn status_code(&self) -> StatusCode {
        match self {
            SubscriptionConfirmError::ValidationError(_) => StatusCode::BAD_REQUEST,
            SubscriptionConfirmError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub struct ConfirmTokenError(sqlx::Error);

impl std::fmt::Debug for ConfirmTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl std::fmt::Display for ConfirmTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A database error was encountered while trying to store a subscription token."
        )
    }
}

impl std::error::Error for ConfirmTokenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(parameters, pool))]
pub async fn confirm(
    parameters: web::Query<Parameters>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, SubscriptionConfirmError> {
    let mut transaction = pool
        .begin()
        .await
        .context("failed to acquire a Postgres connection from the pool")?;

    let subscriber_id =
        get_subscriber_id_from_token(&mut transaction, &parameters.subscription_token)
            .await
            .context("failed to fetch subscriber id")?
            .ok_or_else(|| {
                SubscriptionConfirmError::ValidationError("Invalid subscription token".into())
            })?;

    confirm_subscriber(&mut transaction, subscriber_id)
        .await
        .context("failed to confirm subscriber")?;

    transaction
        .commit()
        .await
        .context("failed to commit transaction")?;

    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(name = "Fetch subscriber_id for given token", skip(transaction, token))]
async fn get_subscriber_id_from_token(
    transaction: &mut Transaction<'_, Postgres>,
    token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT subscriber_id
        FROM subscription_tokens
        WHERE subscription_token = $1
        "#,
        token
    )
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(result.map(|r| r.subscriber_id))
}

#[tracing::instrument(
    name = "Set status of subscriber to confirmed",
    skip(transaction, subscriber_id)
)]
async fn confirm_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE subscriptions
        SET status = 'confirmed'
        WHERE id = $1
        "#,
        subscriber_id
    )
    .execute(&mut **transaction)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
