use ratatui::{Terminal, backend::TestBackend};

use crate::interactive::{
    InteractiveApp,
    state::{AppState, InputFocus, Mode},
};
use std::time::Duration;

#[test]
fn test_show_and_clear_message() {
    let mut state = AppState::new(vec![]);
    state.show_message("Test", Duration::from_secs(1));
    assert_eq!(state.message, Some("Test".to_string()));
    state.clear_message();
    assert!(state.message.is_none());
}

#[test]
fn test_add_variable() {
    let mut state = AppState::new(vec![]);
    state.mode = Mode::Add;
    state.input_key = "VAR1".to_string();
    state.input_value = "VALUE1".to_string();
    // Simulate pressing Enter in add mode.
    if !state.input_key.trim().is_empty() {
        state.entries.push((
            state.input_key.trim().to_string(),
            state.input_value.trim().to_string(),
        ));
        state.mode = Mode::List;
    }
    assert_eq!(state.entries.len(), 1);
    assert_eq!(state.entries[0], ("VAR1".to_string(), "VALUE1".to_string()));
}

#[test]
fn test_toggle_input_focus() {
    let mut state = AppState::new(vec![]);
    state.mode = Mode::Add;
    state.input_focus = InputFocus::Key;
    // Simulate pressing Tab.
    state.input_focus = match state.input_focus {
        InputFocus::Key => InputFocus::Value,
        InputFocus::Value => InputFocus::Key,
    };
    assert_eq!(state.input_focus, InputFocus::Value);
}

#[test]
fn test_edit_variable() {
    let mut state = AppState::new(vec![("VAR1".to_string(), "OLD".to_string())]);
    state.mode = Mode::Edit("VAR1".to_string());
    state.input_value = "NEW".to_string();
    if let Mode::Edit(ref key) = state.mode {
        if let Some(entry) = state.entries.iter_mut().find(|(k, _)| k == key) {
            entry.1 = state.input_value.trim().to_string();
            state.mode = Mode::List;
        }
    }
    assert_eq!(state.entries[0], ("VAR1".to_string(), "NEW".to_string()));
}

#[test]
fn test_delete_variable() {
    let mut state = AppState::new(vec![("VAR1".to_string(), "VALUE1".to_string())]);
    state.mode = Mode::Delete("VAR1".to_string());
    state.entries.retain(|(k, _)| k != "VAR1");
    state.mode = Mode::List;
    assert!(state.entries.is_empty());
}

#[test]
fn test_reload() {
    // Start with one entry.
    let mut state = AppState::new(vec![("VAR1".to_string(), "OLD".to_string())]);
    // Inject our custom getter for the test.
    #[cfg(test)]
    {
        state.variable_getter = Some(Box::new(|| {
            vec![
                ("VAR1".to_string(), "NEW".to_string()),
                ("VAR2".to_string(), "VALUE2".to_string()),
            ]
        }));
    }
    state.request_reload();
    state.reload();
    // After reload, we expect exactly 2 entries.
    assert_eq!(state.entries.len(), 2);
    assert_eq!(state.entries[0], ("VAR1".to_string(), "NEW".to_string()));
}

#[test]
fn test_interactive_app_creation() {
    let app = InteractiveApp::new();
    assert!(!app.state.should_quit);
    assert!(!app.state.entries.is_empty());
}

#[test]
fn test_app_quit_state() {
    let mut app = InteractiveApp::new();
    app.state.should_quit = true;

    let backend = TestBackend::new(20, 20);
    let mut terminal = Terminal::new(backend).unwrap();

    let result = app.run(&mut terminal);
    assert!(result.is_ok());
}

#[test]
fn test_state_initialization() {
    let vars = crate::variables::get_variables();
    let state = AppState::new(vars.clone());
    assert_eq!(state.entries, vars);
    assert_eq!(state.scroll_offset, 0);
    assert!(!state.should_quit);
}

#[test]
fn test_handle_list_mode_quit() {
    use crate::interactive::controller::handle_list_mode;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    let mut state = AppState::new(vec![]);
    let key_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
    handle_list_mode(&mut state, key_event);
    assert!(state.should_quit);
}

#[test]
fn test_handle_list_mode_add() {
    use crate::interactive::controller::handle_list_mode;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
    use crate::interactive::controller::handle_list_mode;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    let mut state = AppState::new(vec![("VAR1".to_string(), "VALUE1".to_string())]);
    let key_event = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::empty());
    handle_list_mode(&mut state, key_event);
    assert_eq!(state.mode, Mode::Edit("VAR1".to_string()));
    assert_eq!(state.input_value, "VALUE1".to_string());
    assert_eq!(state.input_cursor_value, 6);
}

#[test]
fn test_handle_list_mode_delete() {
    use crate::interactive::controller::handle_list_mode;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    let mut state = AppState::new(vec![("VAR1".to_string(), "VALUE1".to_string())]);
    let key_event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::empty());
    handle_list_mode(&mut state, key_event);
    assert_eq!(state.mode, Mode::Delete("VAR1".to_string()));
}

#[test]
fn test_handle_list_mode_down() {
    use crate::interactive::controller::handle_list_mode;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    let mut state = AppState::new(vec![
        ("VAR1".to_string(), "VALUE1".to_string()),
        ("VAR2".to_string(), "VALUE2".to_string()),
    ]);
    let key_event = KeyEvent::new(KeyCode::Down, KeyModifiers::empty());
    handle_list_mode(&mut state, key_event);
    assert_eq!(state.current_index, 1);
}

#[test]
fn test_handle_list_mode_up() {
    use crate::interactive::controller::handle_list_mode;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    let mut state = AppState::new(vec![
        ("VAR1".to_string(), "VALUE1".to_string()),
        ("VAR2".to_string(), "VALUE2".to_string()),
    ]);
    state.current_index = 1;
    let key_event = KeyEvent::new(KeyCode::Up, KeyModifiers::empty());
    handle_list_mode(&mut state, key_event);
    assert_eq!(state.current_index, 0);
}

#[test]
fn test_handle_list_mode_reload() {
    use crate::interactive::controller::handle_list_mode;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    let mut state = AppState::new(vec![]);
    let key_event = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL);
    handle_list_mode(&mut state, key_event);
    assert!(state.reload_requested);
}

#[test]
fn test_handle_add_mode_enter() {
    use crate::interactive::controller::handle_add_mode;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    let mut state = AppState::new(vec![]);
    state.mode = Mode::Add;
    state.input_key = "VAR1".to_string();
    state.input_value = "VALUE1".to_string();
    let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
    handle_add_mode(&mut state, key_event);
}

#[test]
fn test_handle_add_mode_enter_empty_key() {
    use crate::interactive::controller::handle_add_mode;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
    use crate::interactive::controller::handle_add_mode;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    let mut state = AppState::new(vec![]);
    state.mode = Mode::Add;
    let key_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
    handle_add_mode(&mut state, key_event);
    assert_eq!(state.mode, Mode::List);
}

#[test]
fn test_handle_add_mode_tab() {
    use crate::interactive::controller::handle_add_mode;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
    use crate::interactive::controller::handle_add_mode;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
    use crate::interactive::controller::handle_add_mode;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
    use crate::interactive::controller::handle_add_mode;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
    use crate::interactive::controller::handle_add_mode;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
    handle_add_mode(&mut state, key_event);
    assert_eq!(state.input_value, "VALUE1".to_string());
    assert_eq!(state.input_cursor_value, 6);
}

#[test]
fn test_handle_edit_mode_enter() {
    use crate::interactive::controller::handle_edit_mode;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    let mut state = AppState::new(vec![("VAR1".to_string(), "OLD".to_string())]);
    state.mode = Mode::Edit("VAR1".to_string());
    state.input_value = "NEW".to_string();
    let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
    handle_edit_mode(&mut state, key_event);
}

#[test]
fn test_handle_edit_mode_esc() {
    use crate::interactive::controller::handle_edit_mode;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    let mut state = AppState::new(vec![("VAR1".to_string(), "OLD".to_string())]);
    state.mode = Mode::Edit("VAR1".to_string());
    let key_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
    handle_edit_mode(&mut state, key_event);
    assert_eq!(state.mode, Mode::List);
}

#[test]
fn test_handle_edit_mode_left() {
    use crate::interactive::controller::handle_edit_mode;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    let mut state = AppState::new(vec![("VAR1".to_string(), "OLD".to_string())]);
    state.mode = Mode::Edit("VAR1".to_string());
    state.input_cursor_value = 1;
    let key_event = KeyEvent::new(KeyCode::Left, KeyModifiers::empty());
    handle_edit_mode(&mut state, key_event);
    assert_eq!(state.input_cursor_value, 0);
}

#[test]
fn test_handle_edit_mode_right() {
    use crate::interactive::controller::handle_edit_mode;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
    use crate::interactive::controller::handle_edit_mode;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
    use crate::interactive::controller::handle_edit_mode;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
    use crate::interactive::controller::handle_delete_mode;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    let mut state = AppState::new(vec![("VAR1".to_string(), "OLD".to_string())]);
    state.mode = Mode::Delete("VAR1".to_string());
    let key_event = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::empty());
    handle_delete_mode(&mut state, key_event);
}

#[test]
fn test_handle_delete_mode_no() {
    use crate::interactive::controller::handle_delete_mode;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
