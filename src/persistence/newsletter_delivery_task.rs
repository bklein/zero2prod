use sqlx::{PgPool, Postgres, Transaction};
use std::fmt::Debug;

use uuid::Uuid;

use super::PgTransaction;

#[tracing::instrument(skip_all)]
pub async fn delete_newsletter_delivery_task(
    mut transaction: PgTransaction<'_>,
    issue_id: Uuid,
    email: &str,
) -> Result<(), anyhow::Error> {
    sqlx::query!(
        r#"
        DELETE FROM issue_delievery_queue
        WHERE
            newsletter_issue_id = $1 AND
            subscriber_email = $2
        "#,
        issue_id,
        email
    )
    .execute(&mut transaction)
    .await?;
    transaction.commit().await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn dequeue_newsletter_delivery_task(
    pool: &PgPool,
) -> Result<Option<(PgTransaction, Uuid, String)>, anyhow::Error> {
    let mut transaction = pool.begin().await?;
    let r = sqlx::query!(
        r#"
        SELECT newsletter_issue_id, subscriber_email
        FROM issue_delievery_queue
        FOR UPDATE
        SKIP LOCKED
        LIMIT 1
        "#
    )
    .fetch_optional(&mut transaction)
    .await?;

    if let Some(r) = r {
        Ok(Some((
            transaction,
            r.newsletter_issue_id,
            r.subscriber_email,
        )))
    } else {
        Ok(None)
    }
}

#[tracing::instrument(skip_all)]
pub async fn enqueue_newsletter_delivery_tasks(
    transaction: &mut Transaction<'_, Postgres>,
    newsletter_issue_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO issue_delievery_queue (
            newsletter_issue_id,
            subscriber_email
        )
        SELECT $1, email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
        newsletter_issue_id
    )
    .execute(transaction)
    .await?;
    Ok(())
}
