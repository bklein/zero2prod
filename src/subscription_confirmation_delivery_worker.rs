use crate::{
    configuration::Settings,
    domain::{NewSubscriber, NewsletterIssue},
    email_client::EmailClient,
    persistence::subscription_confirmation_task::{
        delete_subscription_confirmation_task, dequeue_subscription_confirmation_task_and_parse,
    },
    startup::{get_connection_pool, ApplicationBaseUrl},
};
use anyhow::Context;
use sqlx::PgPool;
use std::time::Duration;

pub enum ExecutionOutcome {
    TaskComplete,
    EmptyQueue,
}

#[tracing::instrument(skip_all)]
pub async fn try_execute_task(
    pool: &PgPool,
    email_client: &EmailClient,
    base_url: &ApplicationBaseUrl,
) -> Result<ExecutionOutcome, anyhow::Error> {
    match dequeue_subscription_confirmation_task_and_parse(pool).await? {
        Some((transaction, task)) => {
            send_confirmation_email(
                email_client,
                task.subscriber,
                &base_url.0,
                &task.confirmation_token,
            )
            .await
            .context("Failed to send confirmation email.")?;
            delete_subscription_confirmation_task(transaction, task.subscriber_id).await?;
            Ok(ExecutionOutcome::TaskComplete)
        }
        None => Ok(ExecutionOutcome::EmptyQueue),
    }
}

async fn worker_loop(
    pool: PgPool,
    email_client: EmailClient,
    base_url: &ApplicationBaseUrl,
) -> Result<(), anyhow::Error> {
    loop {
        match try_execute_task(&pool, &email_client, base_url).await {
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
    let base_url = ApplicationBaseUrl(configuration.application.base_url);
    worker_loop(connection_pool, email_client, &base_url).await
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
    let plain_body = format!(
        "Welcome to our newsletter!\n\
                Visit {} to confirm your subscription.",
        confirmation_link
    );
    let html_body = format!(
        "Welcome to our newsletter!<br />\
                Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );
    let newsletter_issue =
        NewsletterIssue::validate_new("Welcome!".to_owned(), plain_body, html_body)
            .expect("invalid newsletter");
    email_client
        .send_email(&new_subscriber.email, &newsletter_issue)
        .await
}

#[cfg(test)]
mod test {
    use sqlx::{Connection, Executor, PgConnection};
    use uuid::Uuid;
    use wiremock::{
        matchers::{method, path},
        MockServer,
    };
    use wiremock::{Mock, ResponseTemplate};

    use crate::{
        configuration::{DatabaseSettings, Environment},
        domain::{SubscriberEmail, SubscriberName},
        workflows::complete_new_subscriber_workflow,
    };

    use super::*;

    fn get_configuration() -> Result<Settings, config::ConfigError> {
        let base_path = std::env::current_dir().expect("Failed to get current directory.");
        let configuration_directory = base_path.join("configuration");
        let environment: Environment = std::env::var("APP_ENVIRONMENT")
            .unwrap_or_else(|_| "local".into())
            .try_into()
            .expect("Failed to parse APP_ENVIRONMENT");
        let environment_filename = format!("{}.yaml", environment.as_str());
        let settings = config::Config::builder()
            .add_source(config::File::from(
                configuration_directory.join("base.yaml"),
            ))
            .add_source(config::File::from(
                configuration_directory.join(environment_filename),
            ))
            .add_source(
                config::Environment::with_prefix("APP")
                    .prefix_separator("_")
                    .separator("__"),
            )
            .build()?;
        settings.try_deserialize::<Settings>()
    }

    async fn init_db(email_server: &MockServer) -> PgPool {
        let configuration = {
            let mut c = get_configuration().expect("failed to read config");
            c.database.database_name = Uuid::new_v4().to_string();
            c.application.port = 0;
            c.email_client.base_url = email_server.uri();
            c
        };

        configure_database(&configuration.database).await
    }

    async fn configure_database(config: &DatabaseSettings) -> PgPool {
        let mut connection = PgConnection::connect_with(&config.without_db())
            .await
            .expect("Failed to connect to db.");
        connection
            .execute(&*format!(r#"CREATE DATABASE "{}";"#, config.database_name))
            .await
            .expect("Failed to create db.");
        let connection_pool = PgPool::connect_with(config.with_db())
            .await
            .expect("Failed to connect to db.");
        sqlx::migrate!("./migrations")
            .run(&connection_pool)
            .await
            .expect("migrations");
        connection_pool
    }

    #[tokio::test]
    async fn dequeue_task_works_ok() {
        let email_server = MockServer::start().await;
        Mock::given(path("/email"))
            .and(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&email_server)
            .await;

        let pool = init_db(&email_server).await;

        let new_subscriber = NewSubscriber {
            email: SubscriberEmail::parse("test@test.com".to_owned()).unwrap(),
            name: SubscriberName::parse("Joe Test".to_owned()).unwrap(),
        };
        assert!(complete_new_subscriber_workflow(&pool, new_subscriber)
            .await
            .is_ok());

        let task = dequeue_subscription_confirmation_task_and_parse(&pool)
            .await
            .expect("db problem");
        assert!(task.is_some());
        let (transaction, task) = task.unwrap();
        let r = delete_subscription_confirmation_task(transaction, task.subscriber_id).await;
        assert!(r.is_ok());

        let task = dequeue_subscription_confirmation_task_and_parse(&pool)
            .await
            .expect("db problem");
        assert!(task.is_none());
    }
}
