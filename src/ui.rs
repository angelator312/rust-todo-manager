use std::cmp::min;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Row, Table, TableState, Wrap},
};

use crate::app::{App, CurrentScreen, CurrentlyEditing};

pub const DIALOG_STYLE: Style = Style::new().bg(Color::Black).fg(Color::White);
const DIALOG_TITLE: Style = Style::new().bg(Color::Black).fg(Color::Yellow);
const DIALOG_TEXT: Style = Style::new().bg(Color::Black).fg(Color::White);
pub const DIALOG_EDITOR_ACTIVE_TAB: Style = Style::new().bg(Color::LightYellow).fg(Color::White);
const HELP_TEXT_STYLE: Style = Style::new().bg(Color::White).fg(Color::Red);
const TODO_TEXT_STYLE: Style = Style::new().fg(Color::Yellow);
const ACTIVE_TODO_TEXT_STYLE: Style = Style::new().fg(Color::Blue);
const MAIN_TITLE_TEXT_STYLE: Style = Style::new().fg(Color::Green);

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}

fn render_popup(frame: &mut Frame, title: &str, str: String) {
    frame.render_widget(Clear, frame.area()); //this clears the entire screen and anything already drawn
    let popup_block = Block::default()
        .title(title)
        .borders(Borders::NONE)
        .style(DIALOG_TITLE);

    let exit_text = Text::styled(str, DIALOG_TEXT);
    // the `trim: false` will stop the text from being cut off when over the edge of the block
    let exit_paragraph = Paragraph::new(exit_text)
        .block(popup_block)
        .wrap(Wrap { trim: false });

    let area = centered_rect(60, 25, frame.area());
    frame.render_widget(exit_paragraph, area);
}

pub fn ui(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.area());
    let title_block = Block::default().borders(Borders::ALL);

    let title = Paragraph::new(Text::styled(
        String::from("Todo Manager : ") + &app.path_to_now_todo.join("/"),
        MAIN_TITLE_TEXT_STYLE,
    ))
    .block(title_block);

    frame.render_widget(title, chunks[0]);
    let mut rows = Vec::<Row>::new();
    let (term_cols, _) = crossterm::terminal::size().unwrap(); // T
    let perc_for_todo_type = (1700.0 / term_cols as f64).ceil().floor() as u16;
    let widths = [
        Constraint::Percentage(100 - perc_for_todo_type),
        Constraint::Percentage(perc_for_todo_type),
    ];
    let w = (((100 - perc_for_todo_type) as f64 / 100.0) * term_cols as f64).floor() as usize;
    for (_idx, key) in app.todos[&app.id_of_now_root]
        .children
        .clone()
        .iter()
        .enumerate()
    {
        let mut str: String =
            app.todos[key].text.clone()[..min(w - 3, app.todos[key].text.len())].into();
        if app.todos[key].text.len() > w - 3 {
            str.push_str("...");
        }
        rows.push(Row::new([
            str,
            app.todos[key].todo_type.clone().to_string(),
        ]));
    }

    let table = Table::new(rows, widths)
        // .header(header)
        // .footer(footer.italic())
        .column_spacing(1)
        .style(TODO_TEXT_STYLE)
        .row_highlight_style(ACTIVE_TODO_TEXT_STYLE);
    // .cell_highlight_style(Style::new().reversed().yellow())
    let mut table_state: TableState = TableState::default();
    table_state.select(Some(app.idx_of_now_selected));
    frame.render_stateful_widget(table, chunks[1], &mut table_state);
    let current_keys_hint = {
        match app.current_screen {
            CurrentScreen::Main => Span::styled(
                "(q/s) to quit or save /(l) to load / (n) to make new pair/(d) to delete",
                HELP_TEXT_STYLE,
            ),
            CurrentScreen::Editing => Span::styled(
                "(ESC) to cancel/(Tab) to switch boxes/enter to complete",
                HELP_TEXT_STYLE,
            ),
            CurrentScreen::Exiting { for_quit: _ } | CurrentScreen::Loading => Span::styled(
                "(enter) to confirm the operation, (ESC) to cancel",
                HELP_TEXT_STYLE,
            ),
            CurrentScreen::Deleting => Span::styled(
                "(ESC) to cancel/(Tab) to switch boxes, are you sure you want to delete?",
                HELP_TEXT_STYLE,
            ),
        }
    };

    let key_notes_footer =
        Paragraph::new(Line::from(current_keys_hint)).block(Block::default().borders(Borders::ALL));
    frame.render_widget(key_notes_footer, chunks[2]);
    if let Some(editing) = &app.currently_editing {
        let popup_block = Block::default()
            .title("Enter a new todo")
            .borders(Borders::NONE)
            .style(DIALOG_STYLE);

        let area = centered_rect(60, 25, frame.area());
        frame.render_widget(popup_block, area);
        let popup_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
            .split(area);
        let mut key_block = Block::default().title("Text").borders(Borders::ALL);
        let mut value_block = Block::default().title("Value").borders(Borders::ALL);

        match editing {
            CurrentlyEditing::TodoText => key_block = key_block.style(DIALOG_EDITOR_ACTIVE_TAB),
            CurrentlyEditing::TodoType => value_block = value_block.style(DIALOG_EDITOR_ACTIVE_TAB),
        };

        // let mut key_text = app.textarea;
        // key_text.set_block(key_block);
        // .block(key_block)
        // .wrap(Wrap { trim: true });
        frame.render_widget(&app.textarea, popup_chunks[0]);

        let value_text = Paragraph::new(app.todo_type.to_string()).block(value_block);
        frame.render_widget(value_text, popup_chunks[1]);
    }
    let for_quit_str = if let CurrentScreen::Exiting { for_quit } = app.current_screen {
        match for_quit {
            true => "Quit",
            false => "Save as",
        }
    } else if let CurrentScreen::Loading = app.current_screen {
        "Loading"
    } else {
        "-1"
    };
    if let CurrentScreen::Exiting { for_quit: _ } | CurrentScreen::Loading = app.current_screen {
        let str: String = match app.current_screen {
            CurrentScreen::Loading => {
                "Would you like to load a todo list?(enter to load,escape to abort)\n"
            }
            CurrentScreen::Exiting { for_quit: _ } => {
                "Would you like to save a todo list?(enter to save,escape to abort)\n"
            }
            _ => "",
        }
        .into();
        render_popup(frame, for_quit_str, str + &app.text_input);
    } else if let CurrentScreen::Deleting = app.current_screen {
        let str: String = "Type 'y' so we can be sure\n".into();
        render_popup(frame, "Y/N", str + &app.text_input);
    }
}
