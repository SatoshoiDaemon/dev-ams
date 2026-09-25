pub trait TerminalRenderer {
    fn render(&mut self, text: &str);
}
pub struct PlainTerminal;
impl TerminalRenderer for PlainTerminal {
    fn render(&mut self, text: &str) {
        println!("{text}");
    }
}

impl PlainTerminal {
    pub fn read_line(&mut self) -> std::io::Result<Option<String>> {
        use std::io::Write;
        print!("> ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        let read = std::io::stdin().read_line(&mut input)?;
        Ok((read > 0).then_some(input))
    }
}
