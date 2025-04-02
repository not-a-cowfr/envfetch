use crate::interactive::state::{AppState, InputFocus, Mode};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::io;
use std::time::Duration;

pub fn handle_input(state: &mut AppState) -> io::Result<()> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key_event) = event::read()? {
            if key_event.kind == KeyEventKind::Press {
                match state.mode.clone() {
                    Mode::List => handle_list_mode(state, key_event),
                    Mode::Add => handle_add_mode(state, key_event),
                    Mode::Edit(_) => handle_edit_mode(state, key_event),
                    Mode::Delete(_) => handle_delete_mode(state, key_event),
                }
            }
        }
    }

    if state.reload_requested {
        state.reload();
    }

    if let Some(expiry) = state.message_expiry {
        if std::time::Instant::now() > expiry {
            state.clear_message();
        }
    }
    Ok(())
}

fn handle_list_mode(state: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.should_quit = true;
        }
        KeyCode::Char('a') => {
            state.mode = Mode::Add;
            state.input_key.clear();
            state.input_value.clear();
            state.input_cursor_key = 0;
            state.input_cursor_value = 0;
            state.input_focus = InputFocus::Key;
        }
        KeyCode::Char('e') => {
            if let Some((k, v)) = state.entries.get(state.current_index) {
                state.mode = Mode::Edit(k.clone());
                state.input_value = v.clone();
                state.input_cursor_value = state.input_value.len();
            }
        }
        KeyCode::Char('d') => {
            if let Some((k, _)) = state.entries.get(state.current_index) {
                state.mode = Mode::Delete(k.clone());
            }
        }
        KeyCode::Down => {
            if state.current_index < state.entries.len().saturating_sub(1) {
                state.current_index += 1;
                let visible = 10;
                if state.current_index >= state.scroll_offset + visible {
                    state.scroll_offset += 1;
                }
            }
        }
        KeyCode::Up => {
            if state.current_index > 0 {
                state.current_index -= 1;
                if state.current_index < state.scroll_offset {
                    state.scroll_offset = state.current_index;
                }
            }
        }
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.request_reload();
        }
        _ => {}
    }
}

fn handle_add_mode(state: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            if !state.input_key.trim().is_empty() {
                state.entries.push((
                    state.input_key.trim().to_string(),
                    state.input_value.trim().to_string(),
                ));
                state.show_message("Variable added", Duration::from_secs(2));
                state.mode = Mode::List;
            } else {
                state.show_message("Key cannot be empty", Duration::from_secs(2));
            }
        }
        KeyCode::Esc => state.mode = Mode::List,
        KeyCode::Tab => {
            state.input_focus = match state.input_focus {
                InputFocus::Key => InputFocus::Value,
                InputFocus::Value => InputFocus::Key,
            };
        }
        KeyCode::Left => match state.input_focus {
            InputFocus::Key => {
                if state.input_cursor_key > 0 {
                    state.input_cursor_key -= 1;
                }
            }
            InputFocus::Value => {
                if state.input_cursor_value > 0 {
                    state.input_cursor_value -= 1;
                }
            }
        },
        KeyCode::Right => match state.input_focus {
            InputFocus::Key => {
                if state.input_cursor_key < state.input_key.len() {
                    state.input_cursor_key += 1;
                }
            }
            InputFocus::Value => {
                if state.input_cursor_value < state.input_value.len() {
                    state.input_cursor_value += 1;
                }
            }
        },
        KeyCode::Backspace => match state.input_focus {
            InputFocus::Key => {
                if state.input_cursor_key > 0 {
                    state.input_key.remove(state.input_cursor_key - 1);
                    state.input_cursor_key -= 1;
                }
            }
            InputFocus::Value => {
                if state.input_cursor_value > 0 {
                    state.input_value.remove(state.input_cursor_value - 1);
                    state.input_cursor_value -= 1;
                }
            }
        },
        KeyCode::Char(c) => match state.input_focus {
            InputFocus::Key => {
                state.input_key.insert(state.input_cursor_key, c);
                state.input_cursor_key += 1;
            }
            InputFocus::Value => {
                state.input_value.insert(state.input_cursor_value, c);
                state.input_cursor_value += 1;
            }
        },
        _ => {}
    }
}

fn handle_edit_mode(state: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            if let Mode::Edit(ref key_name) = state.mode {
                if let Some(entry) = state.entries.iter_mut().find(|(k, _)| k == key_name) {
                    entry.1 = state.input_value.trim().to_string();
                    state.show_message("Variable updated", Duration::from_secs(2));
                }
                state.mode = Mode::List;
            }
        }
        KeyCode::Esc => state.mode = Mode::List,
        KeyCode::Left => {
            if state.input_cursor_value > 0 {
                state.input_cursor_value -= 1;
            }
        }
        KeyCode::Right => {
            if state.input_cursor_value < state.input_value.len() {
                state.input_cursor_value += 1;
            }
        }
        KeyCode::Backspace => {
            if state.input_cursor_value > 0 {
                state.input_value.remove(state.input_cursor_value - 1);
                state.input_cursor_value -= 1;
            }
        }
        KeyCode::Char(c) => {
            state.input_value.insert(state.input_cursor_value, c);
            state.input_cursor_value += 1;
        }
        _ => {}
    }
}

fn handle_delete_mode(state: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Char('y') => {
            if let Mode::Delete(ref key_name) = state.mode {
                state.entries.retain(|(k, _)| k != key_name);
                state.show_message("Variable deleted", Duration::from_secs(2));
            }
            state.mode = Mode::List;
        }
        KeyCode::Char('n') | KeyCode::Esc => state.mode = Mode::List,
        _ => {}
    }
}
