use actix_web::{http::header::ContentType, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;
use std::fmt::Write;

pub async fn get_newsletters_form(
    flast_message: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    let mut error_html = String::new();
    for m in flast_message.iter() {
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
    <form action="/admin/newsletters" method="post">
        <label>Newsletter Title
        <br />
        <input
            type="text"
            placeholder="Newsletter title"
            name="title"
          >
          </label>
      <br />
      <label>Newsletter HTML content
      <br />
          <textarea
              placeholder="Enter newsletter content"
              name="html"
            ></textarea>
      </label>
      <br />
      <label>Newsletter text content
      <br />
          <textarea
              placeholder="Enter newsletter content"
              name="text"
            ></textarea>
      </label>
      <br />
      <button type="submit">Send newsletter</button>
    </form>
  </body>
</html>"#
        )))
}
