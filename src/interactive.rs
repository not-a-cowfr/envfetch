pub mod controller;
pub mod state;
#[cfg(test)]
pub mod tests;
pub mod view;

use crate::variables; // Function to get environment variables.
use ratatui::{Terminal, backend::Backend};
use std::io;

pub struct InteractiveApp {
    state: state::AppState,
}

impl InteractiveApp {
    pub fn new() -> Self {
        Self {
            state: state::AppState::new(variables::get_variables()),
        }
    }

    pub fn run<B>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()>
    where
        B: Backend,
    {
        while !self.state.should_quit {
            #[cfg(test)]
            {
                self.state.should_quit = true;
            }
            terminal.draw(|f| view::render(&self.state, f))?;
            // Handle input (this may update scrolling, reload, etc.)
            controller::handle_input(&mut self.state)?;
        }
        Ok(())
    }
}
