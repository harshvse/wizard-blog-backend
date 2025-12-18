use actix_web::{HttpResponse, Responder, web};
use chrono::Utc;
use sqlx::PgPool;
use tracing::Instrument;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

pub async fn subscribe(form: web::Form<FormData>, db_pool: web::Data<PgPool>) -> impl Responder {
    let request_id = Uuid::new_v4();

    let request_span = tracing::info_span!(
        "Adding new subscriber details in the database",
        %request_id,
        Subscriber_Name = %form.name,
        Subscriber_Email = %form.email
    );

    let _request_span_guard = request_span.enter();

    let query_span = tracing::info_span!("Adding new subscriber details in the database");

    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
    "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(db_pool.get_ref())
    .instrument(query_span)
    .await
    {
        Ok(_) => {
            tracing::info!("New subscriber details have been saved");
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error!("failed to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
