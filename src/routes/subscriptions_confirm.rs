use crate::persistence::subscriber::{confirm_subscriber, get_subscriber_id_from_token};
use actix_web::http::StatusCode;
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
        None => Err(ConfirmError::InvalidToken),
        Some(subscriber_id) => {
            confirm_subscriber(&pool, subscriber_id)
                .await
                .context("Failed to update db")?;
            Ok(HttpResponse::Ok().finish())
        }
    }
}

#[derive(Debug)]
pub struct InvalidToken {}

impl std::error::Error for InvalidToken {}

impl std::fmt::Display for InvalidToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "The token was invalid or there was no associated user.")
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfirmError {
    #[error("Invalid confirmation token")]
    InvalidToken,
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for ConfirmError {
    fn status_code(&self) -> StatusCode {
        match self {
            ConfirmError::InvalidToken => StatusCode::UNAUTHORIZED,
            ConfirmError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
