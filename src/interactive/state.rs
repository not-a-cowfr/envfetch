use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    List,
    Add,
    Edit(String),   // Holds the key being edited.
    Delete(String), // Holds the key to be deleted.
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputFocus {
    Key,
    Value,
}

#[cfg(test)]
pub type VariableGetter = Option<Box<dyn Fn() -> Vec<(String, String)>>>;

pub struct AppState {
    pub mode: Mode,
    pub should_quit: bool,
    pub entries: Vec<(String, String)>,
    pub current_index: usize,
    pub scroll_offset: usize,
    pub message: Option<String>,
    pub message_expiry: Option<Instant>,
    // Input buffers for add/edit modes.
    pub input_key: String,
    pub input_value: String,
    // Cursor positions for the input fields.
    pub input_cursor_key: usize,
    pub input_cursor_value: usize,
    // Which field is currently focused.
    pub input_focus: InputFocus,
    // Flag to indicate a reload request.
    pub reload_requested: bool,
    // Optional variable getter override for testing.
    #[cfg(test)]
    pub variable_getter: VariableGetter,
}

impl AppState {
    pub fn new(entries: Vec<(String, String)>) -> Self {
        Self {
            mode: Mode::List,
            should_quit: false,
            entries,
            current_index: 0,
            scroll_offset: 0,
            message: None,
            message_expiry: None,
            input_key: String::new(),
            input_value: String::new(),
            input_cursor_key: 0,
            input_cursor_value: 0,
            input_focus: InputFocus::Key,
            reload_requested: false,
            #[cfg(test)]
            variable_getter: None,
        }
    }

    /// Show a temporary message.
    pub fn show_message(&mut self, msg: &str, duration: Duration) {
        self.message = Some(msg.to_string());
        self.message_expiry = Some(Instant::now() + duration);
    }

    /// Clear the message.
    pub fn clear_message(&mut self) {
        self.message = None;
        self.message_expiry = None;
    }

    /// Request a reload of the variable list.
    pub fn request_reload(&mut self) {
        self.reload_requested = true;
    }

    /// Reload the list of variables.
    ///
    /// In production this calls `crate::variables::get_variables()`,
    /// but if `variable_getter` is set (e.g. in tests) it will use that.
    pub fn reload(&mut self) {
        #[cfg(test)]
        {
            if let Some(ref getter) = self.variable_getter {
                self.entries = getter();
            } else {
                self.entries = crate::variables::get_variables();
            }
        }
        #[cfg(not(test))]
        {
            self.entries = crate::variables::get_variables();
        }
        self.current_index = 0;
        self.scroll_offset = 0;
        self.reload_requested = false;
        self.show_message("List reloaded", Duration::from_secs(2));
    }
}
