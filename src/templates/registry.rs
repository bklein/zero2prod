use handlebars::Handlebars;
use serde::Serialize;
use std::path::{Path, PathBuf};

fn template_root<P: AsRef<Path>>(paths: &[P]) -> PathBuf {
    let mut path = PathBuf::from("./src/templates/");
    for p in paths {
        path.push(p);
    }
    path
}

pub struct TemplateRegistry<'reg>(Handlebars<'reg>);

impl<'reg> TemplateRegistry<'reg> {
    pub fn render<T>(&self, name: &str, data: &T) -> String
    where
        T: Serialize,
    {
        self.0
            .render(name, data)
            .expect("Failed to render template")
    }
}

pub fn register_templates<'reg>() -> TemplateRegistry<'reg> {
    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_file(
            "password",
            &template_root(&["admin", "password", "get.hbs"]),
        )
        .expect("Failed to load template");
    TemplateRegistry(handlebars)
}
