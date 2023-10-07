use crate::templates::render_password_template;
use actix_web::{http::header::ContentType, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;

pub async fn change_password_form(
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    let msgs: Vec<&str> = flash_messages.iter().map(|m| m.content()).collect();
    let body = render_password_template(&msgs);

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}
