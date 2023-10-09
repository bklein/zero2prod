use crate::idempotency::IdempotencyKey;

use super::{GlobalContext, TemplateRegistry};

pub fn render_password_template(
    template_registry: &TemplateRegistry,
    global_context: &GlobalContext,
) -> String {
    template_registry.render_with_default_layout("password", "Change Password", global_context)
}

pub fn render_newsletters_template(
    template_registry: &TemplateRegistry,
    global_context: &GlobalContext,
    idempotency_key: IdempotencyKey,
) -> String {
    let data = serde_json::json!({"idempotency_key": idempotency_key});
    template_registry.render_data_with_default_layout(
        "newsletters",
        "Create newsletter",
        global_context,
        &data,
    )
}

pub fn render_admin_dashboard(
    template_registry: &TemplateRegistry,
    global_context: &GlobalContext,
    username: &str,
) -> String {
    let data = serde_json::json!({"username": username});
    template_registry.render_data_with_default_layout(
        "admin_dashboard",
        "Admin Dashboard",
        global_context,
        &data,
    )
}
