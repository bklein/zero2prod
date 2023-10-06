use crate::{
    session_state::TypedSession,
    utils::{e500, see_other},
};
use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use secrecy::{ExposeSecret, Secret};

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_confirmation: Secret<String>,
}

pub async fn change_password(
    form: web::Form<FormData>,
    session: TypedSession,
) -> Result<HttpResponse, actix_web::Error> {
    if session.get_user_id().map_err(e500)?.is_none() {
        return Ok(see_other("/login"));
    }
    if form.new_password.expose_secret() != form.new_password_confirmation.expose_secret() {
        FlashMessage::error("Password does not match confirmation.").send();
        return Ok(see_other("/admin/password"));
    }
    todo!()
}
