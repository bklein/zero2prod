use crate::{
    domain::{NewsletterIssue},
};
use sqlx::{PgPool, Postgres, Transaction};
use std::fmt::Debug;


use uuid::Uuid;

#[derive(Debug)]
struct RawNewsletterIssue {
    title: String,
    text_content: String,
    html_content: String,
}

#[tracing::instrument(skip_all)]
pub async fn fetch_newsletter_issue(
    pool: &PgPool,
    issue_id: Uuid,
) -> Result<NewsletterIssue, anyhow::Error> {
    let issue = sqlx::query_as!(
        RawNewsletterIssue,
        r#"
        SELECT title, text_content, html_content
        FROM newsletter_issues
        WHERE
            newsletter_issue_id = $1
        "#,
        issue_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(
        NewsletterIssue::validate_new(issue.title, issue.text_content, issue.html_content)
            .expect("invalid newsletter stored in database"),
    )
}

#[tracing::instrument(skip_all)]
pub async fn insert_newsletter_issue(
    transaction: &mut Transaction<'_, Postgres>,
    newsletter_issue: &NewsletterIssue,
) -> Result<Uuid, sqlx::Error> {
    let newsletter_issue_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO newsletter_issues (
            newsletter_issue_id,
            title,
            text_content,
            html_content,
            published_at
        )
        VALUES ($1, $2, $3, $4, now())
        "#,
        newsletter_issue_id,
        newsletter_issue.title(),
        newsletter_issue.text(),
        newsletter_issue.html()
    )
    .execute(transaction)
    .await?;
    Ok(newsletter_issue_id)
}
