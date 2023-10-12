#[derive(Debug)]
pub struct NewsletterIssue {
    title: String,
    text_content: String,
    html_content: String,
}

impl NewsletterIssue {
    pub fn validate_new(
        title: String,
        text_content: String,
        html_content: String,
    ) -> Result<Self, Vec<&'static str>> {
        let mut validation_msgs = vec![];
        if title.is_empty() {
            validation_msgs.push("The newsletter must have a title.");
        }
        if text_content.is_empty() {
            validation_msgs.push("The newsletter must have text content.");
        }
        if html_content.is_empty() {
            validation_msgs.push("The newsletter must have HTML content.");
        }

        if validation_msgs.is_empty() {
            Ok(Self {
                title,
                text_content,
                html_content,
            })
        } else {
            Err(validation_msgs)
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn text(&self) -> &str {
        &self.text_content
    }

    pub fn html(&self) -> &str {
        &self.html_content
    }
}
