use super::{registry::TemplateRegistry, GlobalContext};

pub fn render_login_template(
    template_registry: &TemplateRegistry,
    global_context: &GlobalContext,
) -> String {
    template_registry.render_with_default_layout("login", "Login", global_context)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::templates::assert_and_get_element;
    use crate::templates::register_templates;
    use scraper::Html;

    #[test]
    fn can_render_login_form() {
        let html = render_login_template(&register_templates(), &GlobalContext::default());
        let html = Html::parse_document(&html);
        let form = assert_and_get_element(&html.root_element(), "form");
        assert_eq!(form.value().attr("action"), Some("/login"));
        assert_eq!(form.value().attr("method"), Some("post"));
    }
}
