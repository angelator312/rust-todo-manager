use nanoid::nanoid;
use ratatui::widgets::{Block, Borders};
use ratatui_textarea::TextArea;
use serde::{Deserialize, Serialize};

use crate::notify_error;
use crate::todo::{Todo03, TodoNode};
use crate::{config::Config, notifications, todo::Todo};
use std::collections::BTreeMap;
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
pub type SaveStruct = SaveStruct04; // last SaveStruct
enum VersionedData {
    V01(BTreeMap<usize, Todo02>),
    V02(SaveStruct02),
    V03(SaveStruct03),
    V04(SaveStruct04),
    V05(SaveStruct04),
}
#[derive(Deserialize, Serialize)]
pub struct SaveStruct03 {
    pub todos: BTreeMap<String, Todo03>,
    pub version: String,
}

#[derive(Deserialize, Serialize)]
pub struct SaveStruct04 {
    pub todos: BTreeMap<String, Todo>,
    pub tree: BTreeMap<String, TodoNode>,
    pub version: String,
}

#[derive(Deserialize, Serialize)]
pub struct SaveStruct02 {
    pub todos: BTreeMap<usize, Todo02>,
    pub version: String,
}

#[derive(Deserialize, Serialize)]
pub struct Todo02 {
    pub todo_type: String,
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
    pub todo_type: TextArea<'static>,
    pub all_todo_types:Vec<String>,
    pub text_input: String,
    pub current_screen: CurrentScreen, // the current screen the user is looking at, and will later determine what is rendered.
    pub currently_editing: Option<CurrentlyEditing>,
    pub id_of_now_root: Id,
    pub idx_of_now_selected: usize,
    pub id_of_now_editing: Id,
    pub todos: BTreeMap<Id, Todo>,
    pub tree: BTreeMap<Id, TodoNode>,
    pub is_new: bool,
    pub config: Config,
    pub loaded_file: String,
    pub path_to_now_todo: Vec<String>,
    pub textarea: TextArea<'static>,
}
const VERSION_NOW: &str = "0.5";
impl App {
    pub fn new() -> App {
        let mut hash = BTreeMap::new();
        let mut tree = BTreeMap::new();
        let root_id: String = "0".to_owned();
        hash.insert(root_id.clone(), Todo::make_root());
        tree.insert(root_id.clone(), TodoNode::make_root());
        App {
            text_input: String::new(),
            all_todo_types:vec!["Todo: Todo".into(),"Todo: Done".into(),"Todo: WIP".into()],
            todo_type: TextArea::new(vec!["Todo: Todo".into()]),
            current_screen: CurrentScreen::Main,
            currently_editing: None,
            id_of_now_root: root_id.clone(),
            id_of_now_editing: root_id,
            idx_of_now_selected: 0,
            todos: hash,
            tree,
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
        self.todo_type = TextArea::new(vec!["Todo: Todo".into()]);
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
        self.todo_type = self.todos[&id].todo_type.lines().collect();
        self.id_of_now_editing = id.clone();
        self.textarea.clear();
        self.textarea.insert_str(self.text_input.clone());
        self.textarea.move_cursor(ratatui_textarea::CursorMove::Top);
        self.textarea
            .move_cursor(ratatui_textarea::CursorMove::Head);
    }
    pub fn save_todo(&mut self) {
        if self.is_new {
            self.tree.insert(
                self.id_of_now_editing.clone(),
                TodoNode::new(
                    self.id_of_now_editing.clone(),
                    self.id_of_now_root.clone(),
                    vec![],
                ),
            );
            // self.id_of_now_root.clone(),
            self.todos.insert(
                self.id_of_now_editing.clone(),
                Todo::new(
                    "".into(),
                    "Todo: Todo".into(),
                    self.id_of_now_editing.clone(),
                ),
            );
        }
        if let Some(refer) = self.todos.get_mut(&self.id_of_now_editing) {
            refer.text = self.textarea.lines().join("\n");
            refer.todo_type = self.todo_type.lines().join("\n");
        }
        if self.is_new {
            self.tree
                .get_mut(&self.id_of_now_root)
                .unwrap()
                .children
                .push(self.id_of_now_editing.clone());
        }
        self.text_input = String::new();
        self.todo_type = TextArea::new(vec!["Todo: Todo".into()]);
        self.currently_editing = None;
    }
    pub fn toggle_editing(&mut self) {
        if let Some(edit_mode) = &self.currently_editing {
            match edit_mode {
                CurrentlyEditing::TodoText => {
                    self.currently_editing = Some(CurrentlyEditing::TodoType)
                }
                CurrentlyEditing::TodoType => {
                    self.currently_editing = Some(CurrentlyEditing::TodoText)
                }
            };
        } else {
            self.textarea
                .set_block(Block::default().title("Text").borders(Borders::ALL));
            self.currently_editing = Some(CurrentlyEditing::TodoText);
        }
    }

    pub(crate) fn get_id_of_now_selected(&self) -> Option<Id> {
        if self.tree[&self.id_of_now_root].children.is_empty() {
            None
        } else {
            Some(self.tree[&self.id_of_now_root].children[self.idx_of_now_selected].clone())
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
            tree: self.tree.clone(),
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
        self.id_of_now_root = "0".to_owned(); //root
        self.idx_of_now_selected = 0;
        let ver = detect_version(&contents)?;
        let mut data = load_v(contents, ver)?;
        if let VersionedData::V01(save) = data {
            data = VersionedData::V02(migrate_from_01_to_02(save));
        }
        if let VersionedData::V02(save) = data {
            data = VersionedData::V03(migrate_from_02_to_03(save));
        }
        if let VersionedData::V03(save) = data {
            data = VersionedData::V04(migrate_from_03_to_04(save));
        }
        if let VersionedData::V04(save) = data {
            data = VersionedData::V05(migrate_from_04_to_05(save));
        }
        if let VersionedData::V05(save) = data {
            self.todos = save.todos;
            self.tree = save.tree;
        }
        Ok(())
    }

    pub(crate) fn delete_now_todo(&mut self) {
        let id = &self.get_id_of_now_selected().unwrap();
        if !self.tree.get(id).unwrap().children.is_empty() {
            notifications::warning(
                "Cannot delete",
                "This todo has subtodos. Delete them first.",
            );
            return;
        }
        self.tree
            .get_mut(&self.id_of_now_root)
            .unwrap()
            .children
            .remove(self.idx_of_now_selected);
        if self.idx_of_now_selected > 0 {
            self.idx_of_now_selected -= 1
        }
        self.todos.remove(id);
        self.tree.remove(id);
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
fn load_v(contents: String, v: String) -> Result<VersionedData, String> {
    match &v[..] {
        "0.1" => {
            let a = serde_json::from_str::<BTreeMap<usize, Todo02>>(&contents);
            if let Ok(o) = a {
                Ok(VersionedData::V01(o))
            } else {
                Err("V01 real?".into())
            }
        }
        "0.2" => {
            let a = serde_json::from_str::<SaveStruct02>(&contents);
            if let Ok(o) = a {
                Ok(VersionedData::V02(o))
            } else {
                Err("V02 real?".into())
            }
        }
        "0.3" => {
            let a = serde_json::from_str::<SaveStruct03>(&contents);
            if let Ok(o) = a {
                Ok(VersionedData::V03(o))
            } else {
                Err("V03 real?".into())
            }
        }
        "0.4" => {
            let a = serde_json::from_str::<SaveStruct04>(&contents);
            if let Ok(o) = a {
                Ok(VersionedData::V04(o))
            } else {
                Err("V04 real?".into())
            }
        }
        "0.5" => {
            let a = serde_json::from_str::<SaveStruct04>(&contents);
            if let Ok(o) = a {
                Ok(VersionedData::V05(o))
            } else {
                Err("V05 real?".into())
            }
        }
        _ => Err("Version is corrupted".into()),
    }
}
fn migrate_from_01_to_02(contents: BTreeMap<usize, Todo02>) -> SaveStruct02 {
    SaveStruct02 {
        todos: contents,
        version: "0.2".into(),
    }
}
fn migrate_from_02_to_03(contents: SaveStruct02) -> SaveStruct03 {
    SaveStruct03 {
        todos: (contents
            .todos
            .into_iter()
            .map(|(k, v)| {
                let mut todo = Todo03 {
                    text: v.text,
                    todo_type: v.todo_type,
                    parent: v.parent.to_string(),
                    id: v.id.to_string(),
                    children: vec![],
                };
                todo.children = v.children.into_iter().map(|e| e.to_string()).collect();
                return (k.to_string(), todo);
            })
            .collect()),
        version: "0.3".into(),
    }
}
fn migrate_from_03_to_04(contents: SaveStruct03) -> SaveStruct04 {
    SaveStruct04 {
        todos: contents
            .todos
            .iter()
            .map(|f| {
                (
                    f.0.clone(),
                    Todo::new(f.1.text.clone(), f.1.todo_type.clone(), f.1.id.clone()),
                )
            })
            .collect(),
        tree: contents
            .todos
            .iter()
            .map(|f| {
                (
                    f.0.clone(),
                    TodoNode::new(f.1.id.clone(), f.1.parent.clone(), f.1.children.clone()),
                )
            })
            .collect(),
        version: "0.4".into(),
    }
}
fn migrate_from_04_to_05(contents: SaveStruct04) -> SaveStruct04 {
    SaveStruct04 {
        todos: contents
            .todos
            .iter()
            .map(|f| {
                return (
                    f.0.clone(),
                    Todo {
                        id: f.1.id.clone(),
                        text: f.1.text.clone(),
                        todo_type: match &f.1.todo_type[..] {
                            "Done" => "Todo: Done".into(),
                            "Todo" => "Todo: Todo".into(),
                            "WorkInProgress" => "Todo: WIP".into(),
                            _ => "PROBLEM".into(),
                        },
                    },
                );
            })
            .collect(),
        tree: contents.tree,
        version: "0.5".into(),
    }
}
