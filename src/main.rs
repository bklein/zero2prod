use std::fmt::{Debug, Display};
use tokio::task::JoinError;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::Application;
use zero2prod::telemetry::{get_subscriber, init_subscriber};
use zero2prod::{issue_delivery_worker, subscription_confirmation_delivery_worker};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    let configuration = get_configuration().expect("Could not get config.");
    let application = Application::build(configuration.clone()).await?;
    let application_task = tokio::spawn(application.run_until_stopped());
    let issue_delivery_worker_task = tokio::spawn(issue_delivery_worker::run_worker_until_stopped(
        configuration.clone(),
    ));
    let confirmation_delivery_worker_task = tokio::spawn(
        subscription_confirmation_delivery_worker::run_worker_until_stopped(configuration.clone()),
    );

    tokio::select! {
        o = application_task => report_exit("API", o),
        o = issue_delivery_worker_task => report_exit("Newsletter delivery worker", o),
        o = confirmation_delivery_worker_task => report_exit("Confirmation delivery worker", o),
    }

    Ok(())
}

fn report_exit(task_name: &str, outcome: Result<Result<(), impl Debug + Display>, JoinError>) {
    match outcome {
        Ok(Ok(())) => {
            tracing::info!("{} has exited", task_name)
        }
        Ok(Err(e)) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} failed",
                task_name
            );
        }
        Err(e) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} task failed to complete",
                task_name
            );
        }
    }
}
