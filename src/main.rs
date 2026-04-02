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
        if do_print {
            app.print_json()?;
        }
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
                    KeyCode::Char('e') => {
                        app.current_screen = CurrentScreen::Editing;
                        app.currently_editing = Some(CurrentlyEditing::TodoText);
                    }
                    KeyCode::Char('q') => {
                        app.current_screen = CurrentScreen::Exiting;
                    }
                    KeyCode::Down => {
                        if let Some(last) = app.path_to_selected.last().cloned() {
                            // cloned() takes ownership, reference is released
                            if app.root.children.len() > last + 1 {
                                app.path_to_selected.pop();
                                app.path_to_selected.push(last + 1);
                            }
                        }
                    }
                    KeyCode::Up => {
                        if let Some(last) = app.path_to_selected.last().cloned() {
                            // cloned() takes ownership, reference is released
                            if last>0 {
                                app.path_to_selected.pop();
                                app.path_to_selected.push(last - 1);
                            }
                        }
                    }
                    _ => {}
                },
                CurrentScreen::Exiting => match key.code {
                    KeyCode::Char('y') => {
                        return Ok(true);
                    }
                    KeyCode::Char('n') | KeyCode::Char('q') => {
                        return Ok(false);
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
                _ => {}
            }
        }
    }
}
