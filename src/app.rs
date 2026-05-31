use ratatui::widgets::{Block, Borders};
use ratatui_textarea::TextArea;
use serde::{Deserialize, Serialize};

use crate::notify_error;
use crate::{
    config::Config,
    notifications,
    todo::{Todo, TodoTypes},
    ui::{DIALOG_EDITOR_ACTIVE_TAB, DIALOG_STYLE},
};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
};
pub enum CurrentScreen {
    Main,
    Editing,
    Exiting { for_quit: bool },
    Loading,
    Deleting,
}
pub enum CurrentlyEditing {
    TodoText,
    TodoType,
}
type Id = usize;
pub type Version = String;
#[derive(Deserialize, Serialize)]
pub struct SaveStruct {
    pub todos: HashMap<Id, Todo>,
    pub version: String,
}
pub struct App {
    pub todo_type: TodoTypes,          // the currently being edited json key.
    pub text_input: String,            // the currently being edited json key.
    pub current_screen: CurrentScreen, // the current screen the user is looking at, and will later determine what is rendered.
    pub currently_editing: Option<CurrentlyEditing>,
    pub id_of_now_root: usize,
    pub idx_of_now_selected: usize,
    pub id_of_now_editing: usize,
    pub todos: HashMap<Id, Todo>,
    pub is_new: bool,
    pub config: Config,
    pub loaded_file: String,
    pub path_to_now_todo: Vec<String>,
    pub textarea: TextArea<'static>,
    pub version: Version,
}
const VERSION_NOW: &str = "0.2";
impl App {
    pub fn new() -> App {
        let mut hash = HashMap::new();
        hash.insert(0, Todo::make_root());
        App {
            text_input: String::new(),
            todo_type: TodoTypes::Todo,
            // todos: vec![],
            current_screen: CurrentScreen::Main,
            currently_editing: None,
            id_of_now_root: 0,
            id_of_now_editing: 0,
            idx_of_now_selected: 0,
            todos: hash,
            is_new: false,
            config: match Config::load() {
                Ok(it) => it,
                Err(_) => Config {
                    projects: { HashMap::new() },
                },
            },
            loaded_file: String::new(),
            path_to_now_todo: vec![],
            textarea: TextArea::default(),
            version: VERSION_NOW.into(),
        }
    }
    pub fn start_edit_of_todo(&mut self, id: usize, is_new: bool) {
        self.is_new = is_new;
        self.current_screen = CurrentScreen::Editing;
        self.currently_editing = Some(CurrentlyEditing::TodoText);
        if is_new {
            self.text_input = "".into();
            self.todo_type = TodoTypes::Todo;
            self.id_of_now_editing = 0;
        } else {
            self.text_input = self.todos[&id].text.clone();
            self.todo_type = self.todos[&id].todo_type.clone();
            self.id_of_now_editing = id.clone();
        }
        self.textarea.clear();
        self.textarea.insert_str(self.text_input.clone());
        self.textarea.move_cursor(ratatui_textarea::CursorMove::Top);
        self.textarea
            .move_cursor(ratatui_textarea::CursorMove::Head);
    }
    pub fn save_todo(&mut self) {
        if self.is_new {
            self.id_of_now_editing = match self.todos[&0].text[10..].parse() {
                Ok(n) => n,
                Err(e) => {
                    notifications::error(
                        "Internal error",
                        &format!("Root todo counter is corrupted: {}", e),
                    );
                    return;
                }
            };
            self.todos.get_mut(&0).unwrap().text.truncate(10);
            self.todos
                .get_mut(&0)
                .unwrap()
                .text
                .push_str(&(self.id_of_now_editing + 1).to_string()[..]);
            self.todos.insert(
                self.id_of_now_editing,
                Todo::new(
                    "".into(),
                    TodoTypes::Todo,
                    self.id_of_now_root,
                    self.id_of_now_editing,
                ),
            );
        }
        if let Some(refer) = self.todos.get_mut(&self.id_of_now_editing) {
            refer.text = self.textarea.lines().join("\n");
            refer.todo_type = self.todo_type.clone();
        }
        if self.is_new {
            self.todos
                .get_mut(&self.id_of_now_root)
                .unwrap()
                .children
                .push(self.id_of_now_editing);
        }
        self.text_input = String::new();
        self.todo_type = TodoTypes::Todo;
        self.currently_editing = None;
    }
    pub fn toggle_editing(&mut self) {
        if let Some(edit_mode) = &self.currently_editing {
            match edit_mode {
                CurrentlyEditing::TodoText => {
                    self.textarea.set_style(DIALOG_STYLE);
                    self.currently_editing = Some(CurrentlyEditing::TodoType)
                }
                CurrentlyEditing::TodoType => {
                    self.textarea.set_style(DIALOG_EDITOR_ACTIVE_TAB);
                    self.currently_editing = Some(CurrentlyEditing::TodoText)
                }
            };
        } else {
            self.textarea
                .set_block(Block::default().title("Text").borders(Borders::ALL));
            self.textarea.set_style(DIALOG_EDITOR_ACTIVE_TAB);
            self.currently_editing = Some(CurrentlyEditing::TodoText);
        }
    }

    pub(crate) fn switch_to_next_type(&mut self) {
        self.todo_type = self.todo_type.next();
    }
    pub(crate) fn switch_to_prev_type(&mut self) {
        self.todo_type = self.todo_type.prev();
    }
    pub(crate) fn get_id_of_now_selected(&self) -> Option<usize> {
        if self.todos[&self.id_of_now_root].children.len() == 0 {
            None
        } else {
            Some(self.todos[&self.id_of_now_root].children[self.idx_of_now_selected])
        }
    }
    pub(crate) fn resolve_path(&self, str: String) -> Result<String, String> {
        if str.starts_with("$") {
            if let Some(s) = self.config.get_project(&str[..]) {
                Ok(s.clone())
            } else {
                Err("No Project ".to_owned() + &str)
            }
        } else {
            Ok(str)
        }
    }
    pub(crate) fn save(&mut self, str: String) -> Result<(), String> {
        let str = self
            .resolve_path(str)
            .map_err(|e| notify_error!("Path error", "{}", e))?;
        self.loaded_file = str.clone();
        let json = serde_json::to_string(&SaveStruct {
            todos: self.todos.clone(),
            version: VERSION_NOW.into(),
        })
        .map_err(|e| notify_error!("Save failed", "Failed to serialize todos: {}", e))?;
        let mut file = File::create(&str)
            .map_err(|e| notify_error!("Save failed", "Could not create file '{}': {}", str, e))?;
        file.write_all(json.as_bytes())
            .map_err(|e| notify_error!("Save failed", "Could not write to '{}': {}", str, e))?;
        Ok(())
    }
    pub(crate) fn load(&mut self, str: String) -> Result<(), String> {
        let raw_path = str.clone();
        let str = self.resolve_path(str).map_err(|e| {
            let _ =
                notifications::warning("Path error", &format!("Could not resolve '{}'", raw_path));
            e
        })?;
        self.loaded_file = str.clone();
        let contents = fs::read_to_string(&str)
            .map_err(|e| notify_error!("Load failed", "Could not read '{}': {}", str, e))?;
        self.id_of_now_root = 0; //root
        self.idx_of_now_selected = 0;
        let todos = serde_json::from_str::<SaveStruct>(&contents);
        if let Ok(ths) = todos {
            self.todos = ths.todos;
            self.version = ths.version;
        } else {
            let todos = serde_json::from_str::<HashMap<usize, Todo>>(&contents).map_err(|e| {
                notify_error!("Load failed", "'{}' is not a valid todo file: {}", str, e)
            })?;
            self.todos = todos;
        }
        Ok(())
    }

    pub(crate) fn delete_now_todo(&mut self) {
        let id = &self.get_id_of_now_selected().unwrap();
        self.todos
            .get_mut(&self.id_of_now_root)
            .unwrap()
            .children
            .remove(self.idx_of_now_selected);
        if self.idx_of_now_selected > 0 {
            self.idx_of_now_selected -= 1
        }
        self.todos.remove(id);
    }
}
