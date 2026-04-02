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
    pub root: Todo, // The representation of our key and value pairs with serde Serialize support
    pub current_screen: CurrentScreen, // the current screen the user is looking at, and will later determine what is rendered.
    pub currently_editing: Option<CurrentlyEditing>,
    pub path_to_selected:Vec<usize>
}
impl App {
    fn test_todos() -> Todo {
        let mut a=Todo::new("Root".into(),TodoTypes::Done);
        a.children=vec![
            Todo::new("Let's start".into(), TodoTypes::Done),
            Todo::new("More".into(), TodoTypes::Todo),
            Todo::new("Really random".into(), TodoTypes::WorkInProgress),
            Todo::new("Let's start".into(), TodoTypes::Done),
        ];
        return a;
    }
    pub fn new() -> App {
        App {
            text_input: String::new(),
            todo_type: TodoTypes::Todo,
            root: App::test_todos(),
            // todos: vec![],
            current_screen: CurrentScreen::Main,
            currently_editing: None,
            path_to_selected:vec![0]
        }
    }
    pub fn save_todo(&mut self) {
        self.root.children.push(Todo::new(
            self.text_input.to_owned(),
            self.todo_type.clone(),
        ));

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
        let out = &self.root;
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
