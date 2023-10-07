use super::registry::register_templates;

pub fn render_password_template(msgs: &[&str]) -> String {
    let handlebars = register_templates();
    handlebars.render("password", &serde_json::json!({"msgs": &msgs}))
}
