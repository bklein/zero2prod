use crate::persistence::get_username;
use crate::session_state::TypedSession;
use crate::templates::{render_admin_dashboard, GlobalContext, TemplateRegistry};
use crate::utils::e500;
use actix_web::{http::header::ContentType, web, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;
use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn admin_dashboard(
    session: TypedSession,
    flash_messages: IncomingFlashMessages,
    template_registry: web::Data<TemplateRegistry<'_>>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = session
        .get_user_id()
        .map_err(e500)?
        .expect("Logged in but no user_id");
    let username = get_username(user_id, &pool).await.map_err(e500)?;

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(render_admin_dashboard(
            &template_registry,
            &GlobalContext::from_incoming(flash_messages),
            &username,
        )))
}
