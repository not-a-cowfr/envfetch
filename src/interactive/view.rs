use crate::interactive::state::{AppState, InputFocus, Mode};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap};

pub fn render(state: &AppState, f: &mut Frame) {
    let size = f.area();
    // Divide the screen: main area and footer.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(3)].as_ref())
        .split(size);

    match &state.mode {
        Mode::List => {
            let items: Vec<ListItem> = state
                .entries
                .iter()
                .enumerate()
                .map(|(i, (k, v))| {
                    let marker = if i == state.current_index { "> " } else { "  " };
                    let key_field = format!("{:30}", k);
                    let content = format!("{}{}  {}", marker, key_field, v);
                    let style = if i == state.current_index {
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    ListItem::new(Line::from(Span::styled(content, style)))
                })
                .collect();

            let list = List::new(items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Blue))
                    .title("Variables"),
            );
            let mut list_state = ListState::default();
            list_state.select(Some(state.current_index));
            f.render_stateful_widget(list, chunks[0], &mut list_state);
        }
        Mode::Add => {
            let modal = Paragraph::new(vec![
                Line::from(Span::styled(
                    "Add New Variable",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(format!("Key: {}", state.input_key)),
                Line::from(format!("Value: {}", state.input_value)),
                Line::from("Enter=confirm, Esc=cancel, Tab=switch field, ←/→ move cursor"),
            ])
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Blue))
                    .title("Add"),
            )
            .wrap(Wrap { trim: true });
            let area = centered_rect(60, 40, chunks[0]);
            f.render_widget(modal, area);
            // Calculate inner area offset (1,1) due to rounded border.
            match state.input_focus {
                InputFocus::Key => {
                    // "Key: " is 5 characters.
                    let x = area.x + 1 + 5 + state.input_cursor_key as u16;
                    // The first inner line (line 0) is "Add New Variable",
                    // so "Key:" is on inner line 1 => overall y = area.y + 1 + 1.
                    let y = area.y + 2;
                    f.set_cursor_position((x, y));
                }
                InputFocus::Value => {
                    // "Value: " is 7 characters.
                    let x = area.x + 1 + 7 + state.input_cursor_value as u16;
                    // "Value:" is on inner line 2 => overall y = area.y + 1 + 2.
                    let y = area.y + 3;
                    f.set_cursor_position((x, y));
                }
            }
        }
        Mode::Edit(key) => {
            let modal = Paragraph::new(vec![
                Line::from(Span::styled(
                    format!("Editing: {}", key),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(format!("New Value: {}", state.input_value)),
                Line::from("Enter=confirm, Esc=cancel, ←/→ move cursor"),
            ])
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Blue))
                    .title("Edit"),
            )
            .wrap(Wrap { trim: true });
            let area = centered_rect(60, 40, chunks[0]);
            f.render_widget(modal, area);
            // For edit, "New Value: " is 11 characters.
            let x = area.x + 1 + 11 + state.input_cursor_value as u16;
            // "New Value:" is on inner line 1 => overall y = area.y + 1 + 1.
            let y = area.y + 2;
            f.set_cursor_position((x, y));
        }
        Mode::Delete(key) => {
            let modal = Paragraph::new(vec![
                Line::from(Span::styled(
                    format!("Delete: {}", key),
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )),
                Line::from("Confirm deletion? [y]es / [n]o"),
            ])
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Blue))
                    .title("Delete"),
            )
            .wrap(Wrap { trim: true });
            let area = centered_rect(60, 40, chunks[0]);
            f.render_widget(modal, area);
        }
    }

    // Footer: display instructions or a message.
    let footer_text = if let Some(ref msg) = state.message {
        msg.clone()
    } else {
        "Press [a]dd, [e]dit, [d]elete, [Ctrl+r] reload, [Ctrl+q] quit".to_string()
    };
    let footer = Paragraph::new(footer_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(footer, chunks[1]);
}

// Helper: create a centered rectangle using percentage dimensions.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};
    use std::io;
    use std::time::Duration;

    #[test]
    fn test_draw_list_mode() -> io::Result<()> {
        let backend = TestBackend::new(80, 30);
        let state = AppState::new(vec![
            ("VAR1".to_string(), "VALUE1".to_string()),
            ("VAR2".to_string(), "VALUE2".to_string()),
        ]);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| {
            super::render(&state, f);
        })?;
        Ok(())
    }

    #[test]
    fn test_draw_add_mode() -> io::Result<()> {
        let backend = TestBackend::new(80, 30);
        let mut state = AppState::new(vec![]);
        let mut terminal = Terminal::new(backend).unwrap();
        state.mode = Mode::Add;
        state.input_focus = InputFocus::Key;
        state.input_key = "NEW_VAR".to_string();
        state.input_value = "NEW_VALUE".to_string();
        terminal.draw(|f| {
            super::render(&state, f);
        })?;
        Ok(())
    }

    #[test]
    fn test_draw_edit_mode() -> io::Result<()> {
        let backend = TestBackend::new(80, 30);
        let mut state = AppState::new(vec![("VAR1".to_string(), "VALUE1".to_string())]);
        let mut terminal = Terminal::new(backend).unwrap();
        state.mode = Mode::Edit("VAR1".to_string());
        state.input_value = "EDITED_VALUE".to_string();
        terminal.draw(|f| {
            super::render(&state, f);
        })?;
        Ok(())
    }

    #[test]
    fn test_draw_delete_mode() -> io::Result<()> {
        let backend = TestBackend::new(80, 30);
        let mut state = AppState::new(vec![("VAR1".to_string(), "VALUE1".to_string())]);
        let mut terminal = Terminal::new(backend).unwrap();
        state.mode = Mode::Delete("VAR1".to_string());
        terminal.draw(|f| {
            super::render(&state, f);
        })?;
        Ok(())
    }

    #[test]
    fn test_draw_message() -> io::Result<()> {
        let backend = TestBackend::new(80, 30);
        let mut state = AppState::new(vec![]);
        let mut terminal = Terminal::new(backend).unwrap();
        state.show_message("Test message", Duration::from_secs(2));
        terminal.draw(|f| {
            super::render(&state, f);
        })?;
        Ok(())
    }

    #[test]
    fn test_draw_scrolling() -> io::Result<()> {
        let backend = TestBackend::new(80, 30);
        let mut state = AppState::new(
            (0..20)
                .map(|i| (format!("VAR{}", i), format!("VALUE{}", i)))
                .collect(),
        );
        let mut terminal = Terminal::new(backend).unwrap();
        state.current_index = 15;
        state.scroll_offset = 10;
        terminal.draw(|f| {
            super::render(&state, f);
        })?;
        Ok(())
    }
}
