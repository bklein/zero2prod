use crate::{
    configuration::Settings,
    domain::{subscriber_email::SubscriberEmail},
    email_client::EmailClient,
    persistence::{
        delete_newsletter_delivery_task, fetch_newsletter_issue,
        newsletter_delivery_task::dequeue_newsletter_delivery_task,
    },
    startup::get_connection_pool,
};
use sqlx::{PgPool};

use std::time::Duration;
use tracing::{field::display, Span};


pub enum ExecutionOutcome {
    TaskComplete,
    EmptyQueue,
}

#[tracing::instrument(
    skip_all,
    fields(
        newsletter_issue_id=tracing::field::Empty,
        subscriber_email=tracing::field::Empty,
    ),
    err
)]
pub async fn try_execute_task(
    pool: &PgPool,
    email_client: &EmailClient,
) -> Result<ExecutionOutcome, anyhow::Error> {
    let task = dequeue_newsletter_delivery_task(pool).await?;
    if task.is_none() {
        return Ok(ExecutionOutcome::EmptyQueue);
    }
    let (transaction, issue_id, email) = task.unwrap();
    Span::current()
        .record("newsletter_issue_id", &display(issue_id))
        .record("subscriber_email", &display(&email));
    match SubscriberEmail::parse(email.clone()) {
        Ok(email) => {
            let issue = fetch_newsletter_issue(pool, issue_id).await?;
            if let Err(e) = email_client.send_email(&email, &issue).await {
                tracing::error!(
                    error.cause_chain = ?e,
                    error.message = %e,
                    "Failed to deliver issue to a confirmed subscriber, skipping",
                );
            }
        }
        Err(e) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "Skipping a confirmed subscriber because their email was not validated",
            );
        }
    }
    delete_newsletter_delivery_task(transaction, issue_id, &email).await?;
    Ok(ExecutionOutcome::TaskComplete)
}

async fn worker_loop(pool: PgPool, email_client: EmailClient) -> Result<(), anyhow::Error> {
    loop {
        match try_execute_task(&pool, &email_client).await {
            Ok(ExecutionOutcome::EmptyQueue) => {
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
            Ok(ExecutionOutcome::TaskComplete) => {}
            Err(_) => {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

pub async fn run_worker_until_stopped(configuration: Settings) -> Result<(), anyhow::Error> {
    let connection_pool = get_connection_pool(&configuration.database);
    let email_client = configuration.email_client.client();
    worker_loop(connection_pool, email_client).await
}
