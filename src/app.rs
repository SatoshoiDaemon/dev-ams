use crate::{
    engine::GameState,
    terminal::{PlainTerminal, TerminalRenderer},
};
pub struct App {
    pub state: GameState,
    terminal: PlainTerminal,
}
impl App {
    pub fn new(state: GameState) -> Self {
        Self {
            state,
            terminal: PlainTerminal,
        }
    }
    pub fn render_message(&mut self, message: &str) {
        self.terminal.render(message);
    }

    pub fn command(&self, input: &str) -> String {
        match input.trim() {
            "explain last" => self
                .state
                .explanations
                .render_json()
                .map(|value| value.unwrap_or_else(|| "No completed action to explain.".into()))
                .unwrap_or_else(|error| format!("Failed to render explain last: {error}")),
            command => format!("Unknown command: {command}"),
        }
    }
}
