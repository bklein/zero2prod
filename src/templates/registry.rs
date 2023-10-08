use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use handlebars::Handlebars;
use serde::Serialize;
use std::path::{Path, PathBuf};

pub struct TemplateRegistry<'reg>(Handlebars<'reg>);

impl<'reg> TemplateRegistry<'reg> {
    pub fn render_with_default_layout(
        &self,
        name: &str,
        title: &str,
        global_context: &GlobalContext,
    ) -> String {
        let data = serde_json::json!({});
        self.render_data_with_default_layout(name, title, global_context, &data)
    }

    pub fn render_data_with_default_layout<T>(
        &self,
        name: &str,
        title: &str,
        global_context: &GlobalContext,
        data: &T,
    ) -> String
    where
        T: Serialize,
    {
        self.0
            .render(
                "default_layout",
                &serde_json::json!({
                    "title":title,
                    "flash_messages": global_context.flash_messages(),
                    "inner_template": name,
                    "data": data,
                }),
            )
            .expect("Failed to render template")
    }
}

#[derive(Default)]
pub struct GlobalContext {
    flash_messages: FlashMessagePresentation,
}

impl GlobalContext {
    pub fn from_slice(flash_messages: &[&FlashMessage]) -> Self {
        let flash_messages = flash_messages
            .iter()
            .map(|m| m.content().to_owned())
            .collect::<Vec<String>>();
        let flash_messages = FlashMessagePresentation {
            all: flash_messages,
        };
        Self { flash_messages }
    }

    pub fn from_incoming(flash_messages: IncomingFlashMessages) -> Self {
        let flash_messages = flash_messages.iter().collect::<Vec<&FlashMessage>>();
        Self::from_slice(&flash_messages)
    }

    fn flash_messages(&self) -> &FlashMessagePresentation {
        &self.flash_messages
    }
}

#[derive(Default, serde::Serialize)]
pub struct FlashMessagePresentation {
    pub all: Vec<String>,
}

pub fn register_templates<'reg>() -> TemplateRegistry<'reg> {
    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_file(
            "default_layout",
            &template_root(&["layouts", "default.html"]),
        )
        .expect("Failed to load template");
    handlebars
        .register_template_file(
            "admin_dashboard",
            &template_root(&["admin", "dashboard", "get.html"]),
        )
        .expect("Failed to load template");
    handlebars
        .register_template_file(
            "password",
            &template_root(&["admin", "password", "get.html"]),
        )
        .expect("Failed to load template");
    handlebars
        .register_template_file(
            "newsletters",
            &template_root(&["admin", "newsletters", "get.html"]),
        )
        .expect("Failed to load template");
    handlebars
        .register_template_file("login", &template_root(&["login", "get.html"]))
        .expect("Failed to load template");
    handlebars
        .register_template_file(
            "flash_messages",
            &template_root(&["partials", "flash_messages.html"]),
        )
        .expect("Failed to load template");
    TemplateRegistry(handlebars)
}

fn template_root<P: AsRef<Path>>(paths: &[P]) -> PathBuf {
    let mut path = PathBuf::from("./src/templates/");
    for p in paths {
        path.push(p);
    }
    path
}

#[cfg(test)]
mod test {
    use super::*;
    use scraper::Html;

    use crate::templates::assert_and_get_element;

    #[test]
    fn render_flash_messages_ok() {
        let engine = register_templates();

        let data = serde_json::json!( {
            "flash_messages": &FlashMessagePresentation{
                all: vec!["foobar".to_owned()],
            }
        });

        let html = engine.0.render("flash_messages", &data).unwrap();
        let html = Html::parse_fragment(&html);
        dbg!(&data);
        dbg!(&html.html());
        let p = assert_and_get_element(&html.root_element(), "p");
        let i = assert_and_get_element(&p, "i");
        assert_eq!(i.inner_html(), "foobar")
    }
}
