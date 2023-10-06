use crate::session_state::TypedSession;
use crate::utils::{e500, see_other};
use actix_web::{http::header::ContentType, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;
use std::fmt::Write;

pub async fn change_password_form(
    session: TypedSession,
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    if session.get_user_id().map_err(e500)?.is_none() {
        return Ok(see_other("/login"));
    }
    let mut error_html = String::new();
    for m in flash_messages.iter() {
        writeln!(error_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Home</title>
</head>
<body>
    {error_html}
    <form action="/admin/password" method="post">
        <label>Current password
        <input
            type="password"
            placeholder="Enter current password"
            name="current_password"
          >
          </label>
      <br />
      <label>New password
          <input
              type="password"
              placeholder="Enter new password"
              name="new_password"
            >
      </label>
      <br />
      <label>Confirm new password
          <input
              type="password"
              placeholder="Confirm new password"
              name="new_password_confirmation"
            >
      </label>
      <button type="submit">Change password</button>
    </form>
  </body>
</html>"#,
        )))
}
