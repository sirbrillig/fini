use arboard::Clipboard;
use std::fmt;

pub struct TextPayload(pub String);

impl fmt::Display for TextPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)?;
        Ok(())
    }
}

pub struct HtmlPayload {
    pub html: String,
    pub alt: String,
}

impl fmt::Display for HtmlPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.alt)?;
        Ok(())
    }
}

pub enum CopyPayload {
    Text(TextPayload),
    Html(HtmlPayload),
}

pub trait CopyContent: Sized {
    fn combine(self, other: Self) -> Self;
    fn into_payload(self) -> CopyPayload;
}

impl CopyContent for TextPayload {
    fn combine(self, other: Self) -> Self {
        TextPayload(self.0 + "\n" + &other.0)
    }

    fn into_payload(self) -> CopyPayload {
        CopyPayload::Text(self)
    }
}

impl CopyContent for HtmlPayload {
    fn combine(self, other: Self) -> Self {
        HtmlPayload {
            html: format!("{}<br>{}", self.html, other.html),
            alt: format!("{}\n{}", self.alt, other.alt),
        }
    }

    fn into_payload(self) -> CopyPayload {
        CopyPayload::Html(self)
    }
}

pub trait Copier {
    fn copy(&mut self, payload: &CopyPayload) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct ClipboardCopier;

impl Copier for ClipboardCopier {
    fn copy(&mut self, payload: &CopyPayload) -> Result<(), Box<dyn std::error::Error>> {
        let mut clipboard = Clipboard::new()?;
        match payload {
            CopyPayload::Text(text) => clipboard.set_text(&text.0)?,
            CopyPayload::Html(html) => clipboard.set_html(&html.html, Some(&html.alt))?,
        };
        Ok(())
    }
}

#[derive(Default)]
pub struct MockCopier {
    pub text: String,
    pub html: String,
    pub alt: String,
}

impl MockCopier {
    pub fn new() -> Self {
        Self {
            text: "".to_string(),
            html: "".to_string(),
            alt: "".to_string(),
        }
    }
}

impl Copier for MockCopier {
    fn copy(&mut self, payload: &CopyPayload) -> Result<(), Box<dyn std::error::Error>> {
        match payload {
            CopyPayload::Text(text) => self.text = text.to_string(),
            CopyPayload::Html(html) => {
                self.html = html.html.to_string();
                self.alt = html.alt.to_string();
            }
        };
        Ok(())
    }
}
