use super::{GlobalContext, TemplateRegistry};

pub fn render_home_template(
    template_registry: &TemplateRegistry,
    global_context: &GlobalContext,
) -> String {
    template_registry.render_with_default_layout("home", "Home", global_context)
}
