use nanoid::nanoid;
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
use std::collections::{BTreeMap};
use std::{
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
pub type Id = String;
pub type SaveStruct = SaveStruct03; // last SaveStruct
#[derive(Deserialize, Serialize)]
pub struct SaveStruct03 {
    // добре ще пробвам
    pub todos: BTreeMap<String, Todo>,
    pub version: String,
}

#[derive(Deserialize, Serialize)]
pub struct SaveStruct02 {
    pub todos: BTreeMap<usize, Todo02>,
    pub version: String,
}

#[derive(Deserialize, Serialize)]
pub struct Todo02 {
    pub todo_type: TodoTypes,
    pub text: String,
    pub children: Vec<usize>,
    pub parent: usize,
    id: usize,
}

#[derive(Deserialize, Serialize)]
pub struct OnlyVersion {
    pub version: String,
}

pub struct App {
    pub todo_type: TodoTypes,          // the currently being edited json key.
    pub text_input: String,            // the currently being edited json key.
    pub current_screen: CurrentScreen, // the current screen the user is looking at, and will later determine what is rendered.
    pub currently_editing: Option<CurrentlyEditing>,
    pub id_of_now_root: Id,
    pub idx_of_now_selected: usize,
    pub id_of_now_editing: Id,
    pub todos: BTreeMap<Id, Todo>,
    pub is_new: bool,
    pub config: Config,
    pub loaded_file: String,
    pub path_to_now_todo: Vec<String>,
    pub textarea: TextArea<'static>,
}
const VERSION_NOW: &str = "0.3";
impl App {
    pub fn new() -> App {
        let mut hash = BTreeMap::new();
        let root_id: String = "0".to_owned();
        hash.insert(root_id.clone(), Todo::make_root());
        App {
            text_input: String::new(),
            todo_type: TodoTypes::Todo,
            // todos: vec![],
            current_screen: CurrentScreen::Main,
            currently_editing: None,
            id_of_now_root: root_id.clone(),
            id_of_now_editing: root_id,
            idx_of_now_selected: 0,
            todos: hash,
            is_new: false,
            config: match Config::load() {
                Ok(it) => it,
                Err(_) => Config {
                    projects: { BTreeMap::new() },
                },
            },
            loaded_file: String::new(),
            path_to_now_todo: vec![],
            textarea: TextArea::default(),
        }
    }
    pub fn start_edit_of_new_todo(&mut self) {
        self.is_new = true;
        self.current_screen = CurrentScreen::Editing;
        self.currently_editing = Some(CurrentlyEditing::TodoText);
        self.text_input = "".into();
        self.todo_type = TodoTypes::Todo;
        self.id_of_now_editing = nanoid!();
        self.textarea.clear();
        self.textarea.insert_str(self.text_input.clone());
        self.textarea.move_cursor(ratatui_textarea::CursorMove::Top);
        self.textarea
            .move_cursor(ratatui_textarea::CursorMove::Head);
    }

    pub fn start_edit_of_todo(&mut self, id: Id) {
        self.is_new = false;
        self.current_screen = CurrentScreen::Editing;
        self.currently_editing = Some(CurrentlyEditing::TodoText);
        self.text_input = self.todos[&id].text.clone();
        self.todo_type = self.todos[&id].todo_type.clone();
        self.id_of_now_editing = id.clone();
        self.textarea.clear();
        self.textarea.insert_str(self.text_input.clone());
        self.textarea.move_cursor(ratatui_textarea::CursorMove::Top);
        self.textarea
            .move_cursor(ratatui_textarea::CursorMove::Head);
    }
    pub fn save_todo(&mut self) {
        if self.is_new {
            self.todos.insert(
                self.id_of_now_editing.clone(),
                Todo::new(
                    "".into(),
                    TodoTypes::Todo,
                    self.id_of_now_root.clone(),
                    self.id_of_now_editing.clone(),
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
                .push(self.id_of_now_editing.clone());
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
    pub(crate) fn get_id_of_now_selected(&self) -> Option<Id> {
        if self.todos[&self.id_of_now_root].children.len() == 0 {
            None
        } else {
            Some(self.todos[&self.id_of_now_root].children[self.idx_of_now_selected].clone())
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
        let json = serde_json::to_string_pretty(&SaveStruct {
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

    // имаме проблем. тудо
    fn load_v01(&mut self, contents: String) -> Result<(), String> {
        let todos = serde_json::from_str::<BTreeMap<usize, Todo02>>(&contents);
        if let Ok(ths) = todos {
            self.todos = ths
                .into_iter()
                .map(|(k, v)| {
                    let mut todo =
                        Todo::new(v.text, v.todo_type, v.parent.to_string(), v.id.to_string());
                    todo.children = v.children.into_iter().map(|e| e.to_string()).collect();
                    return (k.to_string(), todo);
                })
                .collect();
            Ok(())
        } else {
            Err("V01 isnt real??".into())
        }
    }
    fn load_v02(&mut self, contents: String) -> Result<(), String> {
        let todos = serde_json::from_str::<SaveStruct02>(&contents);
        if let Ok(ths) = todos {
            self.todos = ths
                .todos
                .into_iter()
                .map(|(k, v)| {
                    let mut todo =
                        Todo::new(v.text, v.todo_type, v.parent.to_string(), v.id.to_string());
                    todo.children = v.children.into_iter().map(|e| e.to_string()).collect();
                    return (k.to_string(), todo);
                })
                .collect();
            Ok(())
        } else {
            Err("V02 isnt real??".into())
        }
    }

    fn load_v03(&mut self, contents: String) -> Result<(), String> {
        let todos = serde_json::from_str::<SaveStruct03>(&contents);
        if let Ok(ths) = todos {
            self.todos = ths.todos;
            Ok(())
        } else {
            Err("V03 isnt real??".into())
        }
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
        self.id_of_now_root = "0".to_owned(); //root
        self.idx_of_now_selected = 0;
        if let Ok(ver) = detect_version(&contents) {
            match &ver[..] {
                "0.1" => self.load_v01(contents),
                "0.2" => self.load_v02(contents),
                "0.3" => self.load_v03(contents),
                _ => Err("Not implemented version".into()),
            }
        } else {
            Err("Cant get version".to_string())
        }
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
pub(crate) fn detect_version(contents: &str) -> Result<String, String> {
    if let Ok(meta) = serde_json::from_str::<OnlyVersion>(contents) {
        Ok(meta.version)
    } else if serde_json::from_str::<serde_json::Value>(contents).is_ok() {
        Ok("0.1".into())
    } else {
        Err("File is not valid JSON".into())
    }
}
