use crate::domain::{
    tasks::SubscriptionConfirmationTask, NewSubscriber, SubscriberEmail, SubscriberName,
};

use sqlx::PgPool;

use uuid::Uuid;

use super::PgTransaction;

#[tracing::instrument(skip_all)]
pub async fn insert_subscription_confirmation_task(
    subscriber_id: Uuid,
    pool: &PgPool,
) -> Result<(), anyhow::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscription_confirmation_delivery_queue (subscriber_id)
        VALUES ($1)
        "#,
        subscriber_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn delete_subscription_confirmation_task(
    mut transaction: PgTransaction<'_>,
    user: Uuid,
) -> Result<(), anyhow::Error> {
    let affected = sqlx::query!(
        r#"
        DELETE FROM subscription_confirmation_delivery_queue
        WHERE
            subscriber_id = $1
        "#,
        user,
    )
    .execute(&mut transaction)
    .await?
    .rows_affected();
    transaction.commit().await?;
    dbg!(&user);
    dbg!(affected);
    Ok(())
}

pub async fn dequeue_subscription_confirmation_task_and_parse(
    pool: &PgPool,
) -> Result<Option<(PgTransaction, SubscriptionConfirmationTask)>, anyhow::Error> {
    if let Some((transaction, raw_task)) = dequeue_task(pool).await? {
        let email = SubscriberEmail::parse(raw_task.email.clone());
        if let Err(e) = &email {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "Skipping a confirmed subscriber because their email was not validated",
            );
        }
        let name = SubscriberName::parse(raw_task.name.clone());
        if let Err(e) = &name {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "Skipping a confirmed subscriber because their name was not validated",
            );
        }
        if name.is_ok() && email.is_ok() {
            let new_subscriber = NewSubscriber {
                email: email.unwrap(),
                name: name.unwrap(),
            };
            Ok(Some((
                transaction,
                SubscriptionConfirmationTask {
                    subscriber_id: raw_task.user,
                    subscriber: new_subscriber,
                    confirmation_token: raw_task.token,
                },
            )))
        } else {
            delete_subscription_confirmation_task(transaction, raw_task.user).await?;
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

#[derive(Debug)]
struct RawTask {
    user: Uuid,
    email: String,
    name: String,
    token: String,
}

async fn dequeue_task(pool: &PgPool) -> Result<Option<(PgTransaction, RawTask)>, anyhow::Error> {
    let mut transaction = pool.begin().await?;
    let r = sqlx::query!(
        r#"
        SELECT subscriber_id
        FROM subscription_confirmation_delivery_queue
        FOR UPDATE
        SKIP LOCKED
        LIMIT 1
        "#
    )
    .fetch_optional(&mut transaction)
    .await?;

    if r.is_none() {
        return Ok(None);
    }
    let subscriber_id = r.unwrap().subscriber_id;

    let r = sqlx::query!(
        r#"
        SELECT id, email, name, subscriptions_token
        FROM subscriptions
        JOIN subscriptions_tokens
        ON subscriber_id = subscriptions.id
        WHERE
            id = $1
        LIMIT 1
        "#,
        subscriber_id
    )
    .fetch_optional(&mut transaction)
    .await?;
    if let Some(r) = r {
        Ok(Some((
            transaction,
            RawTask {
                user: r.id,
                email: r.email,
                name: r.name,
                token: r.subscriptions_token,
            },
        )))
    } else {
        Ok(None)
    }
}
