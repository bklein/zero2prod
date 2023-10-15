use crate::{
    domain::{new_subscriber::generate_confirmation_token, NewSubscriber},
    persistence::{insert_subscriber, insert_subscription_confirmation_task, store_token},
};
use anyhow::Context;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

pub async fn complete_new_subscriber_workflow(
    pool: &PgPool,
    new_subscriber: NewSubscriber,
) -> Result<(), anyhow::Error> {
    let mut transaction = pool.begin().await.context("Failed to connect to db pool")?;
    let subscriber_id = insert_subscriber(&mut transaction, &new_subscriber)
        .await
        .context("Failed to insert subscriber into db.")?;
    let confirmation_token = generate_confirmation_token();
    store_token(&mut transaction, subscriber_id, &confirmation_token)
        .await
        .context("Failed to store the confirmation token for a new subscription.")?;
    transaction
        .commit()
        .await
        .context("Failed to commit db transaction.")?;
    insert_subscription_confirmation_task(subscriber_id, &pool)
        .await
        .context("Failed to enqueue subscriber confirmation task")?;
    Ok(())
}
