use crate::templates::{render_newsletters_template, GlobalContext, TemplateRegistry};
use actix_web::{http::header::ContentType, web, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;

pub async fn get_newsletters_form(
    template_registry: web::Data<TemplateRegistry<'_>>,
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(render_newsletters_template(
            &template_registry,
            &GlobalContext::from_incoming(flash_messages),
        )))
}
