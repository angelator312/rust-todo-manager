use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::Terminal;
use ratatui::crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::prelude::{Backend, CrosstermBackend};
use std::error::Error;
use std::io;
mod app;
mod config;
mod help;
mod notifications;
mod todo;
mod ui;
use crate::{
    app::{App, CurrentScreen, CurrentlyEditing},
    ui::ui,
};
fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stderr = io::stderr(); // This is a special case. Normally using stdout is fine
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = App::new();
    if std::env::args().len() > 1 {
        let args_last = std::env::args().last();
        if let Some(arg) = args_last {
            if let Err(e) = app.load(arg.clone()) {
                let _ = notifications::warning("Load", &format!("Could not open '{}': {}", arg, e));
            }
        }
    }
    let res = run_app(&mut terminal, &mut app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Ok(_do_print) = res {
    } else if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}
fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<bool>
where
    io::Error: From<B::Error>,
{
    loop {
        terminal.draw(|f| ui(f, app))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                // Skip events that are not KeyEventKind::Press
                continue;
            }
            match app.current_screen {
                CurrentScreen::Main => match key.code {
                    KeyCode::Char('n') => {
                        app.start_edit_of_new_todo();
                    }
                    KeyCode::Char('e') => {
                        if let Some(id) = app.get_id_of_now_selected() {
                            app.start_edit_of_todo(id);
                        }
                    }
                    KeyCode::Char('q') | KeyCode::Char('s') => {
                        app.current_screen = CurrentScreen::Exiting {
                            for_quit: (key.code == KeyCode::Char('q')),
                        };
                        app.text_input = app.loaded_file.clone();
                    }
                    KeyCode::Char('l') => {
                        app.current_screen = CurrentScreen::Loading;
                        app.text_input = String::from("");
                    }
                    KeyCode::Char('d') => {
                        app.current_screen = CurrentScreen::Deleting;
                        app.text_input = String::from("");
                    }
                    KeyCode::Down => {
                        if key.modifiers.contains(KeyModifiers::ALT) {
                            if app.idx_of_now_selected
                                < app.tree.get(&app.id_of_now_root).unwrap().children.len() - 1
                            {
                                app.tree
                                    .get_mut(&app.id_of_now_root)
                                    .unwrap()
                                    .children
                                    .swap(app.idx_of_now_selected, app.idx_of_now_selected + 1);
                                app.idx_of_now_selected += 1;
                            }
                        } else {
                            if app.tree[&app.id_of_now_root].children.len()
                                > app.idx_of_now_selected + 1
                            {
                                app.idx_of_now_selected += 1;
                            }
                        }
                    }
                    KeyCode::Up => {
                        if key.modifiers.contains(KeyModifiers::ALT) {
                            if app.idx_of_now_selected > 0 {
                                app.tree
                                    .get_mut(&app.id_of_now_root)
                                    .unwrap()
                                    .children
                                    .swap(app.idx_of_now_selected, app.idx_of_now_selected - 1);
                                app.idx_of_now_selected -= 1;
                            }
                        } else {
                            if app.idx_of_now_selected > 0 {
                                app.idx_of_now_selected -= 1;
                            }
                        }
                    }
                    KeyCode::Right => {
                        if let Some(id) = app.get_id_of_now_selected() {
                            app.id_of_now_root = id;
                            app.idx_of_now_selected = 0;
                            app.path_to_now_todo
                                .push(app.todos[&app.id_of_now_root].text.clone());
                        }
                    }
                    KeyCode::Left => {
                        if let Some(a) = app.tree[&app.tree[&app.id_of_now_root].parent]
                            .children
                            .iter()
                            .position(|x| *x == app.tree[&app.id_of_now_root].id)
                        {
                            app.idx_of_now_selected = a;
                        }
                        app.id_of_now_root = app.tree[&app.id_of_now_root].parent.clone();
                        app.path_to_now_todo.pop();
                    }
                    _ => {}
                },
                CurrentScreen::Exiting { for_quit } => match key.code {
                    KeyCode::Enter | KeyCode::Char('s') => {
                        if !app.text_input.is_empty() {
                            if let Ok(_) = app.save(app.text_input.clone()) {
                                app.current_screen = CurrentScreen::Main;
                                if for_quit {
                                    return Ok(true);
                                }
                            }
                        } else {
                            app.current_screen = CurrentScreen::Main;
                            if for_quit {
                                return Ok(true);
                            }
                        }
                    }
                    KeyCode::Char('q') => {
                        if for_quit {
                            return Ok(true);
                        }
                        app.current_screen = CurrentScreen::Main;
                    }
                    KeyCode::Char(value) => {
                        app.text_input.push(value);
                    }
                    KeyCode::Backspace => {
                        app.text_input.pop();
                    }
                    KeyCode::Esc => {
                        app.current_screen = CurrentScreen::Main;
                    }
                    _ => {}
                },
                CurrentScreen::Loading => match key.code {
                    KeyCode::Esc => {
                        app.current_screen = CurrentScreen::Main;
                    }
                    KeyCode::Enter => {
                        let anem = app.load(app.text_input.clone());
                        match anem {
                            Ok(_) => app.current_screen = CurrentScreen::Main,
                            Err(_) => {} //dont do anything its handled somewhere else
                        }
                    }
                    KeyCode::Char(value) => {
                        app.text_input.push(value);
                    }
                    KeyCode::Backspace => {
                        app.text_input.pop();
                    }
                    _ => {}
                },
                CurrentScreen::Editing => match key.code {
                    KeyCode::Enter
                        if matches!(app.currently_editing, Some(CurrentlyEditing::TodoType)) =>
                    {
                        app.save_todo();
                        app.current_screen = CurrentScreen::Main;
                    }
                    KeyCode::Enter
                        if matches!(
                            app.currently_editing,
                            Some(CurrentlyEditing::TodoTypeAutoComplete)
                        ) =>
                    {
                        let pat = app.todo_type.lines().join("");
                        let mut lines_auto_complete: Vec<String> = vec![];
                        for e in (app.all_todo_types).clone() {
                            if e.starts_with(&pat) {
                                lines_auto_complete.push(e);
                            }
                        }
                        app.todo_type.clear();
                        app.todo_type
                            .insert_str(lines_auto_complete[app.idx_of_helper].clone());
                        app.todo_type.move_cursor(ratatui_textarea::CursorMove::Top);
                        app.todo_type
                            .move_cursor(ratatui_textarea::CursorMove::Head);
                    }
                    KeyCode::Esc => {
                        app.current_screen = CurrentScreen::Main;
                        app.currently_editing = None;
                    }
                    KeyCode::Tab => {
                        app.toggle_editing();
                    }
                    KeyCode::Down
                        if matches!(
                            app.currently_editing,
                            Some(CurrentlyEditing::TodoTypeAutoComplete)
                        ) =>
                    {
                        app.idx_of_helper += 1;
                    }
                    KeyCode::Up
                        if matches!(
                            app.currently_editing,
                            Some(CurrentlyEditing::TodoTypeAutoComplete)
                        ) =>
                    {
                        if app.idx_of_helper > 0 {
                            app.idx_of_helper -= 1;
                        }
                    }
                    _ => {
                        if matches!(app.currently_editing, Some(CurrentlyEditing::TodoText)) {
                            app.textarea.input(key);
                        } else if matches!(app.currently_editing, Some(CurrentlyEditing::TodoType))
                        {
                            app.idx_of_helper = 0;
                            app.todo_type.input(key);
                        }
                    }
                },
                CurrentScreen::Deleting => match key.code {
                    KeyCode::Char(e) => app.text_input.push(e),
                    KeyCode::Enter => {
                        if app.text_input == "y" {
                            //delete the todo
                            app.delete_now_todo();
                        }
                        app.current_screen = CurrentScreen::Main;
                    }
                    KeyCode::Backspace => {
                        app.text_input.pop();
                    }
                    KeyCode::Esc => app.current_screen = CurrentScreen::Main,
                    _ => {}
                },
            }
        }
    }
}
