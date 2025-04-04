use crate::{
    app::{App, Widget},
    theme,
    widgets::options::AppOption,
};

use tui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
};

// Draw the file tree with indentation
pub fn draw_file_tree<B: tui::backend::Backend>(f: &mut tui::Frame<B>, app: &mut App, area: Rect) {
    let mut block = Block::default().borders(Borders::ALL).title("File Browser");

    if app.selected_widget == Widget::FileBrowser {
        block = block.border_type(BorderType::Thick)
    }

    let visible_idxs = app.file_browser_state.visible_entries();
    let items: Vec<ListItem> = visible_idxs
        .iter()
        .map(|&idx| {
            let entry = &app.file_browser_state.file_list[idx];
            let indentation = " ".repeat(entry.depth * 2);
            let filename = entry.path.split('/').last().unwrap_or("UNKNOWN");
            let checked_or_not = if entry.excluded { "" } else { "* " };
            ListItem::new(format!("{}{}{}", indentation, checked_or_not, filename))
        })
        .collect();

    // 2. Build the `List` widget
    let list = List::new(items)
        .block(block)
        .style(Style::default().bg(theme::GRAY))
        .highlight_style(
            Style::default()
                .fg(theme::YELLOW) // yellow
                .bg(theme::LIGHT_GREY)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    // 3. Render with the `list_state` to track selection
    f.render_stateful_widget(list, area, &mut app.file_browser_state.list_state);
}

// Draw the prompt editor
pub fn draw_prompt_editor<B: tui::backend::Backend>(f: &mut tui::Frame<B>, app: &App, area: Rect) {
    let mut block = Block::default()
        .borders(Borders::ALL)
        .title("Prompt Editor");
    if app.selected_widget == Widget::PromptEditor {
        block = block.border_type(BorderType::Thick);
    }
    let paragraph = Paragraph::new(app.prompt_editor_state.get_display_text())
        .block(block)
        .style(Style::default().fg(theme::LIGHT_GREEN).bg(theme::GRAY));
    f.render_widget(paragraph, area);
}

pub fn draw_bottom_options<B: tui::backend::Backend>(
    f: &mut tui::Frame<B>,
    app: &App,
    area: tui::layout::Rect,
) {
    let mut block = Block::default()
        .borders(Borders::ALL)
        .title("Options")
        .style(Style::default().bg(theme::GRAY));
    if app.selected_widget == Widget::Options {
        block = block.border_type(BorderType::Thick)
    }

    let spans = vec![
        if app.options_state.selected_option() == AppOption::Send {
            Span::styled(
                "[Send]",
                Style::default()
                    .fg(theme::BLUE)
                    .add_modifier(Modifier::REVERSED),
            )
        } else {
            Span::raw("[Send]")
        },
        Span::raw("  "), // spacing
        if app.options_state.selected_option() == AppOption::Quit {
            Span::styled(
                "[Quit]",
                Style::default()
                    .fg(theme::PURPLE)
                    .add_modifier(Modifier::REVERSED),
            )
        } else {
            Span::raw("[Quit]")
        },
    ];

    let paragraph = Paragraph::new(Spans::from(spans))
        .block(block)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));

    f.render_widget(paragraph, area);
}

pub fn ui<B: tui::backend::Backend>(f: &mut tui::Frame<B>, app: &mut App) {
    // First, split the screen vertically so we can have a thin pane at the bottom
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        // The top pane takes all remaining space (Min), bottom pane has a fixed height of 3
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(f.size());

    // Now, split the top pane horizontally for the file tree and prompt editor
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(vertical_chunks[0]);

    draw_file_tree(f, app, main_chunks[0]);
    draw_prompt_editor(f, app, main_chunks[1]);
    draw_bottom_options(f, app, vertical_chunks[1]);
}
