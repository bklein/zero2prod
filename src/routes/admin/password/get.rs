use crate::templates::render_password_template;
use actix_web::{http::header::ContentType, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;

pub async fn change_password_form(
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(render_password_template(flash_messages)))
}
