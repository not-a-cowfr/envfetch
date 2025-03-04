use super::InteractiveMode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};

pub fn render(state: InteractiveMode, area: Rect, buf: &mut Buffer) {
    // Split screen into main list and value panel
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(6), // Ensure enough space for main content
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(area);

    render_main_list(state.clone(), chunks[0], buf);
    render_value_panel(state, chunks[1], buf);
}

fn render_main_list(state: InteractiveMode, area: Rect, buf: &mut Buffer) {
    // Increase available height by accounting for all UI elements
    let height = area.height.saturating_sub(6) as usize; // Account for borders, titles, and headers
    let scroll_offset = state.scroll_offset;
    let total_options = state.entries.len();

    let mut content: Vec<Line> = vec![];

    // Add column headers
    content.push(Line::from(vec![
        "  ".into(),
        "NAME".blue().bold(),
        " ".repeat(38).into(), // Padding to align with 40% width
        "VALUE".blue().bold(),
    ]));

    // Add separator line under headers
    content.push(Line::from(""));

    // Calculate visible range
    let visible_height = height.saturating_sub(2); // Account for headers
    let end_index = (scroll_offset + visible_height).min(total_options);

    if scroll_offset > 0 {
        content.push(Line::from("↑ More options above".dark_gray()));
    }

    for (index, (name, value)) in state.entries[scroll_offset..end_index].iter().enumerate() {
        let actual_index = index + scroll_offset;
        let is_selected = state.current_index == actual_index;

        // Truncate value if needed
        let truncated_value = if value.len() > state.truncation_len {
            format!("{}...", &value[..state.truncation_len])
        } else {
            value.clone()
        };

        // Pad name to align columns
        let padded_name = format!("{:width$}", name, width = 38);

        let line = if is_selected {
            Line::from(vec![
                "> ".bold(),
                padded_name.bold(),
                truncated_value.bold(),
            ])
        } else {
            Line::from(vec![
                "  ".into(),
                padded_name.into(),
                truncated_value.dark_gray(),
            ])
        };
        content.push(line);
    }

    if end_index < total_options {
        content.push(Line::from("↓ More options below".dark_gray()));
    }

    let title = Line::from(" envfetch Interactive Mode ".bold());
    let instructions = Line::from(vec![
        " Navigation ".into(),
        "<Up>/".blue().bold(),
        "<Down>".blue().bold(),
        " Value Scroll ".into(),
        "<Left>/".blue().bold(),
        "<Right>".blue().bold(),
        " Reload ".into(),
        "<^R>".blue().bold(),
        " Quit ".into(),
        "<^Q> ".blue().bold(),
    ]);

    let block = Block::bordered()
        .title(title.centered())
        .title_bottom(instructions.centered())
        .border_set(border::ROUNDED);

    Paragraph::new(Text::from(content))
        .block(block)
        .render(area, buf);
}

fn render_value_panel(state: InteractiveMode, area: Rect, buf: &mut Buffer) {
    let mut content = vec![];

    if let Some((name, value)) = state.entries.get(state.current_index) {
        let scroll_offset = state.value_scroll_offset.min(value.len());
        let visible_width = area.width.saturating_sub(4) as usize;
        let end_index = (scroll_offset + visible_width).min(value.len());

        let value_part = &value[scroll_offset..end_index];
        content.push(Line::from(vec![
            name.as_str().blue().bold(),
            "=".into(),
            value_part.into(),
        ]));

        // Navigation hints at the bottom
        let mut nav_hints = vec![" Scroll ".into()];
        if scroll_offset > 0 {
            nav_hints.extend(vec!["<Left".blue().bold(), "/".into()]);
        }
        if end_index < value.len() {
            nav_hints.extend(vec!["Right>".blue().bold()]);
        }
        content.push(Line::from(nav_hints).centered());
    }

    let block = Block::bordered()
        .title(" Current Value ".bold())
        .border_set(border::ROUNDED);

    Paragraph::new(Text::from(content))
        .block(block)
        .render(area, buf);
}
