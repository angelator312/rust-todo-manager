use crate::todo::{Todo, TodoTypes};
pub enum CurrentScreen {
    Main,
    Editing,
    Exiting,
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
    pub todos: Vec<Todo>,
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
            idx_of_now_selected: 0,
            todos: vec![Todo::make_root()],
        }
    }
    pub fn save_todo(&mut self) {
        self.todos.push(Todo::new(
            self.text_input.to_owned(),
            self.todo_type.clone(),
            self.id_of_now_root,
        ));
        if let Some(todo) = self.todos.last().cloned() {
            self.todos[self.id_of_now_root].children.push(todo.id())
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
    pub fn print_json(&self) -> serde_json::Result<()> {
        let out = &self.todos;
        println!("{out:?}");
        Ok(())
    }

    pub(crate) fn switch_to_next_type(&mut self) {
        self.todo_type = self.todo_type.next();
    }
    pub(crate) fn switch_to_prev_type(&mut self) {
        self.todo_type = self.todo_type.prev();
    }
}
