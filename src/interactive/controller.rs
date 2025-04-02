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

pub fn handle_list_mode(state: &mut AppState, key: KeyEvent) {
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

pub fn handle_add_mode(state: &mut AppState, key: KeyEvent) {
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

pub fn handle_edit_mode(state: &mut AppState, key: KeyEvent) {
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

pub fn handle_delete_mode(state: &mut AppState, key: KeyEvent) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_handle_list_mode_quit() {
        let mut state = AppState::new(vec![]);
        let key_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
        handle_list_mode(&mut state, key_event);
        assert!(state.should_quit);
    }

    #[test]
    fn test_handle_list_mode_add() {
        let mut state = AppState::new(vec![]);
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty());
        handle_list_mode(&mut state, key_event);
        assert_eq!(state.mode, Mode::Add);
        assert_eq!(state.input_key, "");
        assert_eq!(state.input_value, "");
        assert_eq!(state.input_cursor_key, 0);
        assert_eq!(state.input_cursor_value, 0);
        assert_eq!(state.input_focus, InputFocus::Key);
    }

    #[test]
    fn test_handle_list_mode_edit() {
        let mut state = AppState::new(vec![("VAR1".to_string(), "VALUE1".to_string())]);
        let key_event = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::empty());
        handle_list_mode(&mut state, key_event);
        assert_eq!(state.mode, Mode::Edit("VAR1".to_string()));
        assert_eq!(state.input_value, "VALUE1".to_string());
        assert_eq!(state.input_cursor_value, 6);
    }

    #[test]
    fn test_handle_list_mode_delete() {
        let mut state = AppState::new(vec![("VAR1".to_string(), "VALUE1".to_string())]);
        let key_event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::empty());
        handle_list_mode(&mut state, key_event);
        assert_eq!(state.mode, Mode::Delete("VAR1".to_string()));
    }

    #[test]
    fn test_handle_list_mode_down() {
        let mut state = AppState::new(vec![("VAR1".to_string(), "VALUE1".to_string()), ("VAR2".to_string(), "VALUE2".to_string())]);
        let key_event = KeyEvent::new(KeyCode::Down, KeyModifiers::empty());
        handle_list_mode(&mut state, key_event);
        assert_eq!(state.current_index, 1);
    }

    #[test]
    fn test_handle_list_mode_up() {
        let mut state = AppState::new(vec![("VAR1".to_string(), "VALUE1".to_string()), ("VAR2".to_string(), "VALUE2".to_string())]);
        state.current_index = 1;
        let key_event = KeyEvent::new(KeyCode::Up, KeyModifiers::empty());
        handle_list_mode(&mut state, key_event);
        assert_eq!(state.current_index, 0);
    }

    #[test]
    fn test_handle_list_mode_reload() {
        let mut state = AppState::new(vec![]);
        let key_event = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL);
        handle_list_mode(&mut state, key_event);
        assert!(state.reload_requested);
    }

    #[test]
    fn test_handle_add_mode_enter() {
        let mut state = AppState::new(vec![]);
        state.mode = Mode::Add;
        state.input_key = "VAR1".to_string();
        state.input_value = "VALUE1".to_string();
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
        handle_add_mode(&mut state, key_event);
        assert_eq!(state.entries.len(), 1);
        assert_eq!(state.entries[0], ("VAR1".to_string(), "VALUE1".to_string()));
        assert_eq!(state.mode, Mode::List);
    }

    #[test]
    fn test_handle_add_mode_enter_empty_key() {
        let mut state = AppState::new(vec![]);
        state.mode = Mode::Add;
        state.input_key = "".to_string();
        state.input_value = "VALUE1".to_string();
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
        handle_add_mode(&mut state, key_event);
        assert_eq!(state.entries.len(), 0);
        assert_eq!(state.mode, Mode::Add);
        assert_eq!(state.message, Some("Key cannot be empty".to_string()));
    }

    #[test]
    fn test_handle_add_mode_esc() {
        let mut state = AppState::new(vec![]);
        state.mode = Mode::Add;
        let key_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
        handle_add_mode(&mut state, key_event);
        assert_eq!(state.mode, Mode::List);
    }

    #[test]
    fn test_handle_add_mode_tab() {
        let mut state = AppState::new(vec![]);
        state.mode = Mode::Add;
        state.input_focus = InputFocus::Key;
        let key_event = KeyEvent::new(KeyCode::Tab, KeyModifiers::empty());
        handle_add_mode(&mut state, key_event);
        assert_eq!(state.input_focus, InputFocus::Value);

        state.input_focus = InputFocus::Value;
        handle_add_mode(&mut state, key_event);
        assert_eq!(state.input_focus, InputFocus::Key);
    }

    #[test]
    fn test_handle_add_mode_left() {
        let mut state = AppState::new(vec![]);
        state.mode = Mode::Add;
        state.input_focus = InputFocus::Key;
        state.input_cursor_key = 1;
        let key_event = KeyEvent::new(KeyCode::Left, KeyModifiers::empty());
        handle_add_mode(&mut state, key_event);
        assert_eq!(state.input_cursor_key, 0);

        state.input_focus = InputFocus::Value;
        state.input_cursor_value = 1;
        handle_add_mode(&mut state, key_event);
        assert_eq!(state.input_cursor_value, 0);
    }

    #[test]
    fn test_handle_add_mode_right() {
        let mut state = AppState::new(vec![]);
        state.mode = Mode::Add;
        state.input_focus = InputFocus::Key;
        state.input_key = "VAR1".to_string();
        state.input_cursor_key = 0;
        let key_event = KeyEvent::new(KeyCode::Right, KeyModifiers::empty());
        handle_add_mode(&mut state, key_event);
        assert_eq!(state.input_cursor_key, 1);

        state.input_focus = InputFocus::Value;
        state.input_value = "VALUE1".to_string();
        state.input_cursor_value = 0;
        handle_add_mode(&mut state, key_event);
        assert_eq!(state.input_cursor_value, 1);
    }

    #[test]
    fn test_handle_add_mode_backspace() {
        let mut state = AppState::new(vec![]);
        state.mode = Mode::Add;
        state.input_focus = InputFocus::Key;
        state.input_key = "VAR1".to_string();
        state.input_cursor_key = 4;
        let key_event = KeyEvent::new(KeyCode::Backspace, KeyModifiers::empty());
        handle_add_mode(&mut state, key_event);
        assert_eq!(state.input_key, "VAR".to_string());
        assert_eq!(state.input_cursor_key, 3);

        state.input_focus = InputFocus::Value;
        state.input_value = "VALUE1".to_string();
        state.input_cursor_value = 6;
        handle_add_mode(&mut state, key_event);
        assert_eq!(state.input_value, "VALUE".to_string());
        assert_eq!(state.input_cursor_value, 5);
    }

    #[test]
    fn test_handle_add_mode_char() {
        let mut state = AppState::new(vec![]);
        state.mode = Mode::Add;
        state.input_focus = InputFocus::Key;
        state.input_key = "VAR".to_string();
        state.input_cursor_key = 3;
        let key_event = KeyEvent::new(KeyCode::Char('1'), KeyModifiers::empty());
        handle_add_mode(&mut state, key_event);
        assert_eq!(state.input_key, "VAR1".to_string());
        assert_eq!(state.input_cursor_key, 4);

        state.input_focus = InputFocus::Value;
        state.input_value = "VALUE".to_string();
        state.input_cursor_value = 5;
        let key_event = KeyEvent::new(KeyCode::Char('1'), KeyModifiers::empty());
        handle_add_mode(&mut state, key_event);
        assert_eq!(state.input_value, "VALUE1".to_string());
        assert_eq!(state.input_cursor_value, 6);
    }

    #[test]
    fn test_handle_edit_mode_enter() {
        let mut state = AppState::new(vec![("VAR1".to_string(), "OLD".to_string())]);
        state.mode = Mode::Edit("VAR1".to_string());
        state.input_value = "NEW".to_string();
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
        handle_edit_mode(&mut state, key_event);
        assert_eq!(state.entries[0], ("VAR1".to_string(), "NEW".to_string()));
        assert_eq!(state.mode, Mode::List);
    }

    #[test]
    fn test_handle_edit_mode_esc() {
        let mut state = AppState::new(vec![("VAR1".to_string(), "OLD".to_string())]);
        state.mode = Mode::Edit("VAR1".to_string());
        let key_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
        handle_edit_mode(&mut state, key_event);
        assert_eq!(state.mode, Mode::List);
    }

    #[test]
    fn test_handle_edit_mode_left() {
        let mut state = AppState::new(vec![("VAR1".to_string(), "OLD".to_string())]);
        state.mode = Mode::Edit("VAR1".to_string());
        state.input_cursor_value = 1;
        let key_event = KeyEvent::new(KeyCode::Left, KeyModifiers::empty());
        handle_edit_mode(&mut state, key_event);
        assert_eq!(state.input_cursor_value, 0);
    }

    #[test]
    fn test_handle_edit_mode_right() {
        let mut state = AppState::new(vec![("VAR1".to_string(), "OLD".to_string())]);
        state.mode = Mode::Edit("VAR1".to_string());
        state.input_value = "OLD".to_string();
        state.input_cursor_value = 0;
        let key_event = KeyEvent::new(KeyCode::Right, KeyModifiers::empty());
        handle_edit_mode(&mut state, key_event);
        assert_eq!(state.input_cursor_value, 1);
    }

    #[test]
    fn test_handle_edit_mode_backspace() {
        let mut state = AppState::new(vec![("VAR1".to_string(), "OLD".to_string())]);
        state.mode = Mode::Edit("VAR1".to_string());
        state.input_value = "OLD".to_string();
        state.input_cursor_value = 3;
        let key_event = KeyEvent::new(KeyCode::Backspace, KeyModifiers::empty());
        handle_edit_mode(&mut state, key_event);
        assert_eq!(state.input_value, "OL".to_string());
        assert_eq!(state.input_cursor_value, 2);
    }

    #[test]
    fn test_handle_edit_mode_char() {
        let mut state = AppState::new(vec![("VAR1".to_string(), "OLD".to_string())]);
        state.mode = Mode::Edit("VAR1".to_string());
        state.input_value = "OL".to_string();
        state.input_cursor_value = 2;
        let key_event = KeyEvent::new(KeyCode::Char('D'), KeyModifiers::empty());
        handle_edit_mode(&mut state, key_event);
        assert_eq!(state.input_value, "OLD".to_string());
        assert_eq!(state.input_cursor_value, 3);
    }

    #[test]
    fn test_handle_delete_mode_yes() {
        let mut state = AppState::new(vec![("VAR1".to_string(), "OLD".to_string())]);
        state.mode = Mode::Delete("VAR1".to_string());
        let key_event = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::empty());
        handle_delete_mode(&mut state, key_event);
        assert_eq!(state.entries.len(), 0);
        assert_eq!(state.mode, Mode::List);
    }

    #[test]
    fn test_handle_delete_mode_no() {
        let mut state = AppState::new(vec![("VAR1".to_string(), "OLD".to_string())]);
        state.mode = Mode::Delete("VAR1".to_string());
        let key_event = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::empty());
        handle_delete_mode(&mut state, key_event);
        assert_eq!(state.entries.len(), 1);
        assert_eq!(state.mode, Mode::List);

        state.mode = Mode::Delete("VAR1".to_string());
        let key_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
        handle_delete_mode(&mut state, key_event);
        assert_eq!(state.entries.len(), 1);
        assert_eq!(state.mode, Mode::List);
    }
}
