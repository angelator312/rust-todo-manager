use std::{
    fs::{self, File},
    io::Write,
};

use crate::todo::{Todo, TodoTypes, get_id};
pub enum CurrentScreen {
    Main,
    Editing,
    Exiting,
    Loading,
}

pub enum CurrentlyEditing {
    TodoText,
    TodoType,
}

pub struct App {
    pub todo_type: TodoTypes,          // the currently being edited json key.
    pub text_input: String,            // the currently being edited json key.
    pub current_screen: CurrentScreen, // the current screen the user is looking at, and will later determine what is rendered.
    pub currently_editing: Option<CurrentlyEditing>,
    pub id_of_now_root: usize,
    pub idx_of_now_selected: usize,
    pub id_of_now_editing: usize,
    pub todos: Vec<Todo>,
    pub is_new: bool,
}
impl App {
    pub fn new() -> App {
        App {
            text_input: String::new(),
            todo_type: TodoTypes::Todo,
            // todos: vec![],
            current_screen: CurrentScreen::Main,
            currently_editing: None,
            id_of_now_root: 0,
            id_of_now_editing: 0,
            idx_of_now_selected: 0,
            todos: vec![Todo::make_root()],
            is_new: false,
        }
    }
    pub fn start_edit_of_todo(&mut self, id: usize, is_new: bool) {
        self.is_new = is_new;
        self.current_screen = CurrentScreen::Editing;
        self.currently_editing = Some(CurrentlyEditing::TodoText);
        self.text_input = self.todos[id].text.clone();
        self.todo_type = self.todos[id].todo_type.clone();
        self.id_of_now_editing = id.clone();
    }
    pub fn save_todo(&mut self) {
        self.todos[self.id_of_now_editing] = Todo::new(
            self.text_input.to_owned(),
            self.todo_type.clone(),
            self.id_of_now_root,
            self.id_of_now_editing,
        );
        if self.is_new {
            self.todos[self.id_of_now_root]
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
                    self.currently_editing = Some(CurrentlyEditing::TodoType)
                }
                CurrentlyEditing::TodoType => {
                    self.currently_editing = Some(CurrentlyEditing::TodoText)
                }
            };
        } else {
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
        if self.todos[self.id_of_now_root].children.len() == 0 {
            None
        } else {
            Some(self.todos[self.id_of_now_root].children[self.idx_of_now_selected])
        }
    }
    pub(crate) fn save(&mut self, str: String) -> Result<(), String> {
        if let Ok(json) = serde_json::to_string(&self.todos) {
            if let Ok(mut file) = File::create(str + ".task.json") {
                file.write_all(json.as_bytes());
            }
            Ok(())
        } else {
            Err("Problem".into())
        }
    }
    pub(crate) fn load(&mut self, str: String) -> Result<(), String> {
        if let Ok(contents) = fs::read_to_string(str + &String::from(".task.json")) {
            let todos=serde_json::from_str::<Vec<Todo>>(&contents).unwrap();
            self.todos=todos;
            self.id_of_now_root=0;//root
            self.idx_of_now_selected=0;
        }
        Ok(())
    }
}
