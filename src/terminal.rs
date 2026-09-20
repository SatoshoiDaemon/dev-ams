pub trait TerminalRenderer { fn render(&mut self, text: &str); }
pub struct PlainTerminal;
impl TerminalRenderer for PlainTerminal { fn render(&mut self, text: &str) { println!("{text}"); } }
