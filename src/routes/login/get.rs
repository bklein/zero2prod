use crate::templates::{render_login_template, GlobalContext, TemplateRegistry};
use actix_web::{http::header::ContentType, web, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;

pub async fn login_form(
    template_registry: web::Data<TemplateRegistry<'_>>,
    flash_messages: IncomingFlashMessages,
) -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(render_login_template(
            &template_registry,
            &GlobalContext::from_incoming(flash_messages),
        ))
}
