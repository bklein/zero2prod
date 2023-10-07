use super::registry::register_templates;
use actix_web_flash_messages::IncomingFlashMessages;

pub fn render_password_template(flash_messages: IncomingFlashMessages) -> String {
    let msgs: Vec<&str> = flash_messages.iter().map(|m| m.content()).collect();
    let handlebars = register_templates();
    handlebars.render("password", &serde_json::json!({"msgs": &msgs}))
}
