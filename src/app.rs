use crate::{engine::GameState, terminal::{PlainTerminal, TerminalRenderer}};
pub struct App { pub state: GameState, terminal: PlainTerminal }
impl App { pub fn new(state: GameState) -> Self { Self { state, terminal: PlainTerminal } } pub fn render_message(&mut self, message: &str) { self.terminal.render(message); } }
