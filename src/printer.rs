pub trait Printer {
    fn print(&mut self, text: &str);
}

pub struct StdoutPrinter;

impl Printer for StdoutPrinter {
    fn print(&mut self, text: &str) {
        println!("{}", text);
    }
}

#[derive(Default)]
pub struct MockPrinter {
    pub text: String,
}

impl MockPrinter {
    pub fn new() -> Self {
        Self {
            text: "".to_string(),
        }
    }

    pub fn clear(&mut self) {
        self.text = "".to_string();
    }
}

impl Printer for MockPrinter {
    fn print(&mut self, text: &str) {
        self.text += format!("{}\n", text).as_str();
    }
}
