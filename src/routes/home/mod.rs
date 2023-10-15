use crate::templates::{render_home_template, GlobalContext, TemplateRegistry};
use actix_web::{http::header::ContentType, web, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;

pub async fn home(
    template_registry: web::Data<TemplateRegistry<'_>>,
    flash_messages: IncomingFlashMessages,
) -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(render_home_template(
            &template_registry,
            &GlobalContext::from_incoming(flash_messages),
        ))
}
