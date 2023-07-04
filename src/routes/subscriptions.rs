use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use crate::email_client::EmailClient;
use crate::startup::ApplicationBaseUrl;
use actix_web::{web, HttpResponse};
use chrono::Utc;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;
use actix_web::ResponseError;
use actix_web::http::StatusCode;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(form: FormData) -> Result<NewSubscriber, Self::Error> {
        let name = SubscriberName::parse(form.name)?;
        let email = SubscriberEmail::parse(form.email)?;
        Ok(NewSubscriber { email, name })
    }
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool, email_client, base_url),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
) -> Result<HttpResponse, SubscribeError> {
    let new_subscriber = form.0.try_into()?;
    let mut transaction = pool.begin().await.map_err(SubscribeError::PoolError)?;
    let subscriber_id = insert_subscriber(&mut transaction, &new_subscriber).await.map_err(SubscribeError::InsertSubscriberError)?;
    let confirmation_token = generate_confirmation_token();
    store_token(&mut transaction, subscriber_id, &confirmation_token).await?;
    transaction.commit().await.map_err(SubscribeError::TransactionCommitError)?;
    send_confirmation_email(
        &email_client,
        new_subscriber,
        &base_url.0,
        &confirmation_token,
    )
    .await?;
    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(
    name = "Store confirmation token into database"
    skip(confirmation_token, transaction)
)]
async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    confirmation_token: &str,
) -> Result<(), SubscribeError> {
    sqlx::query!(
        r#"INSERT INTO subscriptions_tokens (subscriptions_token, subscriber_id)
    VALUES ($1, $2)"#,
        confirmation_token,
        subscriber_id
    )
    .execute(transaction)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        SubscribeError::StoreTokenError(StoreTokenError{})
    })?;
    Ok(())
}

#[tracing::instrument(
    name = "Saving new subscriber in db.",
    skip(new_subscriber, transaction)
)]
async fn insert_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    new_subscriber: &NewSubscriber,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at, status)
    VALUES ($1, $2, $3, $4, 'pending_confirmation')
            "#,
        subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(transaction)
    .await
    .map_err(|e| {
        tracing::error!("Failed query: {:?}", e);
        e
    })?;
    Ok(subscriber_id)
}

#[tracing::instrument(
    name = "Send confirmation email to a new subscriber",
    skip(new_subscriber, email_client, base_url, confirmation_token)
)]
async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
    confirmation_token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, confirmation_token
    );
    let plain_body = &format!(
        "Welcome to our newsletter!\n\
                Visit {} to confirm your subscription.",
        confirmation_link
    );
    let html_body = &format!(
        "Welcome to our newsletter!<br />\
                Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );
    email_client
        .send_email(new_subscriber.email, "Welcome!", html_body, plain_body)
        .await
}

fn generate_confirmation_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

#[derive(Debug)]
pub struct StoreTokenError{}

impl std::fmt::Display for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "There was an error storing the subscription token.")
    }
}

impl std::error::Error for StoreTokenError {}

pub enum SubscribeError {
    ValidationError(String),
    PoolError(sqlx::Error),
    InsertSubscriberError(sqlx::Error),
    TransactionCommitError(sqlx::Error),
    StoreTokenError(StoreTokenError),
    SendEmailError(reqwest::Error),
}

impl std::fmt::Debug for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        error_chain_fmt(self, f)
    }
}

impl std::fmt::Display for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            SubscribeError::InsertSubscriberError(_) => write!(f, "Failed to insert subscriber into db."),
            SubscribeError::PoolError(_) => write!(f, "Failed to connect to db pool."),
            SubscribeError::SendEmailError(_) => write!(f, "Failed to send confirmation email."),
            SubscribeError::StoreTokenError(_) => write!(f, "Failed to store the confirmation token for a new subscription."),
            SubscribeError::TransactionCommitError(_) => write!(f, "Failed to commit db transaction."),
            SubscribeError::ValidationError(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for SubscribeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SubscribeError::InsertSubscriberError(e) => Some(e),
            SubscribeError::PoolError(e) => Some(e),
            SubscribeError::SendEmailError(e) => Some(e),
            SubscribeError::StoreTokenError(e) => Some(e),
            SubscribeError::TransactionCommitError(e) => Some(e),
            SubscribeError::ValidationError(_) => None,
        }
    }
}

impl ResponseError for SubscribeError {
    fn status_code(&self) -> StatusCode {
        match self {
            SubscribeError::ValidationError(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<reqwest::Error> for SubscribeError {
    fn from(e: reqwest::Error) -> Self {
        Self::SendEmailError(e)
    }
}

impl From<StoreTokenError> for SubscribeError {
    fn from(e: StoreTokenError) -> Self {
        Self::StoreTokenError(e)
    }
}

impl From<String> for SubscribeError {
    fn from(e: String) -> Self {
        Self::ValidationError(e)
    }
}

fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}
