use actix_web::ResponseError;
use actix_web::{web, HttpResponse};
use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending registration", skip(parameters, pool))]
pub async fn confirm(
    parameters: web::Query<Parameters>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ConfirmError> {
    let id = get_subscriber_id_from_token(&pool, &parameters.subscription_token)
        .await
        .context("Failed to execute db query.")?;
    match id {
        None => Ok(HttpResponse::Unauthorized().finish()),
        Some(subscriber_id) => {
            confirm_subscriber(&pool, subscriber_id)
                .await
                .context("Invalid confirmation token.")?;
            Ok(HttpResponse::Ok().finish())
        }
    }
}

#[tracing::instrument(name = "Mark subscriber as confirmed", skip(subscriber_id, pool))]
async fn confirm_subscriber(pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' where id = $1"#,
        subscriber_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[tracing::instrument(name = "Get subscriber_id from token", skip(subscription_token, pool))]
async fn get_subscriber_id_from_token(
    pool: &PgPool,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT subscriber_id FROM subscriptions_tokens
        WHERE subscriptions_token = $1"#,
        subscription_token
    )
    .fetch_optional(pool)
    .await?;
    Ok(result.map(|r| r.subscriber_id))
}

#[derive(thiserror::Error)]
pub enum ConfirmError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for ConfirmError {}

impl std::fmt::Debug for ConfirmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "todo")
    }
}
