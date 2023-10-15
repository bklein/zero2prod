use crate::authentication::UserId;
use crate::domain::newsletter_issue::NewsletterIssue;
use crate::idempotency::{save_response, try_processing, IdempotencyKey, NextAction};
use crate::persistence::newsletter_delivery_task::enqueue_newsletter_delivery_tasks;
use crate::persistence::newsletter_issue::insert_newsletter_issue;
use crate::utils::{e400, e500, see_other};
use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(Debug, serde::Deserialize)]
pub struct FormData {
    title: String,
    #[serde(flatten)]
    content: Content,
    idempotency_key: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct Content {
    html: String,
    text: String,
}

#[tracing::instrument(
    name = "Publish a newsletter issue",
    skip_all,
    fields(user_id=%&*user_id)
)]
pub async fn publish_newsletter(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user_id.into_inner();
    let FormData {
        title,
        content: Content {
            html: html_content,
            text: text_content,
        },
        idempotency_key,
    } = form.0;

    let newsletter_issue = match NewsletterIssue::validate_new(title, text_content, html_content) {
        Ok(newsletter_issue) => newsletter_issue,
        Err(validation_msgs) => {
            for m in validation_msgs.iter() {
                FlashMessage::error(m.to_owned()).send();
            }
            return Ok(see_other("/admin/newsletters"));
        }
    };

    let idempotency_key: IdempotencyKey = idempotency_key.try_into().map_err(e400)?;
    let mut transaction = match try_processing(&pool, &idempotency_key, *user_id)
        .await
        .map_err(e500)?
    {
        NextAction::StartProcessing(t) => t,
        NextAction::ReturnSavedResponse(saved_response) => {
            success_message().send();
            return Ok(saved_response);
        }
    };
    let newsletter_issue_id = insert_newsletter_issue(&mut transaction, &newsletter_issue)
        .await
        .context("Failed to store newsletter details")
        .map_err(e500)?;
    enqueue_newsletter_delivery_tasks(&mut transaction, newsletter_issue_id)
        .await
        .context("Failed to enqueue delivery tasks")
        .map_err(e500)?;
    let response = see_other("/admin/dashboard");
    let response = save_response(transaction, &idempotency_key, *user_id, response)
        .await
        .map_err(e500)?;
    success_message().send();
    Ok(response)
}

fn success_message() -> FlashMessage {
    FlashMessage::info("The newsletter has been accepted.")
}
