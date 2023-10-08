mod registry;
pub use registry::*;

mod admin;
pub use admin::*;

mod login;
pub use login::*;

#[cfg(test)]
mod test_utils {
    use scraper::{ElementRef, Selector};

    pub fn assert_and_get_element<'a>(html: &ElementRef<'a>, element: &str) -> ElementRef<'a> {
        let selector = Selector::parse(element);
        assert!(selector.is_ok());
        let selector = selector.unwrap();
        let selection = html.select(&selector).next();
        assert!(selection.is_some());
        selection.unwrap()
    }
}

#[cfg(test)]
use test_utils::assert_and_get_element;
