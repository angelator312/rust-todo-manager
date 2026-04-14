use crossterm::event::{self, Event, KeyCode, KeyEventKind};
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
    let res = run_app(&mut terminal, &mut app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Ok(do_print) = res {
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
                        app.start_edit_of_todo(1, true);
                    }
                    KeyCode::Char('e') => {
                        if let Some(id) = app.get_id_of_now_selected() {
                            app.start_edit_of_todo(id, false);
                        }
                    }
                    KeyCode::Char('q') | KeyCode::Char('s') => {
                        app.current_screen = CurrentScreen::Exiting;
                        app.text_input = String::from("");
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
                        if app.todos[&app.id_of_now_root].children.len()
                            > app.idx_of_now_selected + 1
                        {
                            app.idx_of_now_selected += 1;
                        }
                    }
                    KeyCode::Up => {
                        if app.idx_of_now_selected > 0 {
                            app.idx_of_now_selected -= 1;
                        }
                    }
                    KeyCode::Right => {
                        if let Some(id) = app.get_id_of_now_selected() {
                            app.id_of_now_root = id;
                            app.idx_of_now_selected = 0;
                        }
                    }
                    KeyCode::Left => {
                        if let Some(a) = app.todos[&app.todos[&app.id_of_now_root].parent]
                            .children
                            .iter()
                            .position(|&x| x == app.todos[&app.id_of_now_root].id())
                        {
                            app.idx_of_now_selected = a;
                        }
                        app.id_of_now_root = app.todos[&app.id_of_now_root].parent;
                    }
                    _ => {}
                },
                CurrentScreen::Exiting => match key.code {
                    KeyCode::Enter => {
                        if !app.text_input.is_empty() {
                            app.save(app.text_input.clone());
                        }
                        return Ok(true);
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
                            Err(e) => app.text_input += e.as_str(),
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
                CurrentScreen::Editing if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Enter => {
                        if let Some(editing) = &app.currently_editing {
                            match editing {
                                CurrentlyEditing::TodoText => {
                                    app.currently_editing = Some(CurrentlyEditing::TodoType)
                                }
                                CurrentlyEditing::TodoType => {
                                    app.save_todo();
                                    app.current_screen = CurrentScreen::Main;
                                }
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        if let Some(editing) = &app.currently_editing {
                            match editing {
                                CurrentlyEditing::TodoText => {
                                    app.text_input.pop();
                                }
                                CurrentlyEditing::TodoType => {}
                            }
                        }
                    }
                    KeyCode::Esc => {
                        app.current_screen = CurrentScreen::Main;
                        app.currently_editing = None;
                    }
                    KeyCode::Tab => {
                        app.toggle_editing();
                    }
                    KeyCode::Char(value) => {
                        if let Some(editing) = &app.currently_editing {
                            match editing {
                                CurrentlyEditing::TodoText => {
                                    app.text_input.push(value);
                                }
                                CurrentlyEditing::TodoType => {}
                            }
                        }
                    }
                    KeyCode::Up | KeyCode::Right => {
                        if let Some(editing) = &app.currently_editing {
                            match editing {
                                CurrentlyEditing::TodoText => {}
                                CurrentlyEditing::TodoType => app.switch_to_next_type(),
                            }
                        }
                    }
                    KeyCode::Down | KeyCode::Left => {
                        if let Some(editing) = &app.currently_editing {
                            match editing {
                                CurrentlyEditing::TodoText => {}
                                CurrentlyEditing::TodoType => app.switch_to_prev_type(),
                            }
                        }
                    }

                    _ => {}
                },
                CurrentScreen::Editing => {}
                CurrentScreen::Deleting => match key.code {
                    KeyCode::Char(e) => app.text_input.push(e),
                    KeyCode::Enter => {
                        if app.text_input == "Yes" {
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
