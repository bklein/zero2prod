use anyhow::Context;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::domain::{new_subscriber::generate_confirmation_token, NewSubscriber};

#[derive(Debug)]
pub struct StoreTokenError(sqlx::Error);

impl std::fmt::Display for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "There was an error storing the subscription token.")
    }
}

impl std::error::Error for StoreTokenError {}

#[tracing::instrument(
    name = "Store confirmation token into database"
    skip(confirmation_token, transaction)
)]
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    confirmation_token: &str,
) -> Result<(), StoreTokenError> {
    sqlx::query!(
        r#"INSERT INTO subscriptions_tokens (subscriptions_token, subscriber_id)
    VALUES ($1, $2)"#,
        confirmation_token,
        subscriber_id
    )
    .execute(transaction)
    .await
    .map_err(StoreTokenError)?;
    Ok(())
}
