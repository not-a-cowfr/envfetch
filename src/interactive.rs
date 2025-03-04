mod draw;
mod list;
mod tests;

use std::io;

use super::variables::get_variables;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{DefaultTerminal, Frame};

#[derive(Clone)]
pub enum Mode {
    List,
}

impl Default for Mode {
    fn default() -> Self {
        Self::List
    }
}

#[derive(Clone)]
pub struct InteractiveMode {
    mode: Mode,
    exit: bool,
    entries: Vec<(String, String)>,
    current_index: usize,
    scroll_offset: usize,
    visible_options: usize,
    truncation_len: usize,
    value_scroll_offset: usize, // For horizontal scrolling in value panel
}

impl Default for InteractiveMode {
    fn default() -> Self {
        InteractiveMode {
            mode: Mode::List,
            exit: false,
            entries: get_variables(),
            current_index: 0,
            scroll_offset: 0,
            visible_options: 30,
            truncation_len: 30,
            value_scroll_offset: 0,
        }
    }
}

impl InteractiveMode {
    /// Initialize InteractiveMode
    pub fn init() -> Self {
        Self::default()
    }

    /// Run TUI interface for interactive mode
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    /// Draw TUI
    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    /// Handle events
    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    /// Handle keypresses
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Char('Q')
                if key_event.modifiers == KeyModifiers::CONTROL =>
            {
                self.exit()
            }
            KeyCode::Down => self.down(),
            KeyCode::Up => self.up(),
            KeyCode::Left => self.scroll_value_left(),
            KeyCode::Right => self.scroll_value_right(),
            KeyCode::Char('r') | KeyCode::Char('R')
                if key_event.modifiers == KeyModifiers::CONTROL =>
            {
                self.reload()
            }
            _ => {}
        }
    }

    /// Scroll value left
    fn scroll_value_left(&mut self) {
        if self.value_scroll_offset > 0 {
            self.value_scroll_offset -= 1;
        }
    }

    /// Scroll value right
    fn scroll_value_right(&mut self) {
        if let Some((_, value)) = self.entries.get(self.current_index) {
            if self.value_scroll_offset < value.len() {
                self.value_scroll_offset += 1;
            }
        }
    }

    /// Exit
    fn exit(&mut self) {
        self.exit = true;
    }

    /// Scroll list down
    fn down(&mut self) {
        let max_index = self.entries.len().saturating_sub(1);
        if self.current_index < max_index {
            self.current_index += 1;
            self.value_scroll_offset = 0;

            // Keep a fixed number of items visible before scrolling
            let visible_area = self.visible_options.saturating_sub(8);
            let scroll_trigger = self.scroll_offset + (visible_area.saturating_sub(4));

            // Only scroll when we're past the visible area
            if self.current_index > scroll_trigger {
                self.scroll_offset += 1;
            }
        }
    }

    /// Scroll list up
    fn up(&mut self) {
        if self.current_index > 0 {
            self.current_index -= 1;
            self.value_scroll_offset = 0;

            // Scroll up when cursor moves above current scroll position
            if self.current_index < self.scroll_offset {
                self.scroll_offset = self.current_index;
            }
        }
    }

    /// Reload variables list
    fn reload(&mut self) {
        self.entries = super::variables::get_variables();
        self.current_index = 0;
        self.scroll_offset = 0;
        self.value_scroll_offset = 0;
    }
}
