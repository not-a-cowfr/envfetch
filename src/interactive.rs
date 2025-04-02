pub mod state;
pub mod controller;
pub mod view;
#[cfg(test)]
pub mod tests;

use crate::variables; // Function to get environment variables.
use ratatui::{backend::Backend, Terminal};
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
            terminal.draw(|f| view::render(&self.state, f))?;
            // Handle input (this may update scrolling, reload, etc.)
            controller::handle_input(&mut self.state)?;
        }
        Ok(())
    }
}
