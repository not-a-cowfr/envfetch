use crate::interactive::state::{AppState, InputFocus, Mode};
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
