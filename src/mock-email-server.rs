use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tracing::instrument(skip_all)]
async fn ret_200(req: HttpRequest, data: web::Json<Option<SendEmailRequest>>) -> HttpResponse {
    dbg!(req);
    dbg!(data);
    HttpResponse::Ok().finish()
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest {
    from: String,
    to: String,
    subject: String,
    html_body: String,
    text_body: String,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let subscriber = get_subscriber("mock-email-server".into(), "trace".into(), std::io::stdout);
    init_subscriber(subscriber);

    let address = format!("{}:{}", "localhost", 8008);
    let listener = TcpListener::bind(address)?;
    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/email", web::post().to(ret_200))
    })
    .listen(listener)?
    .run()
    .await?;
    Ok(())
}
