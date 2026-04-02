use std::fmt::Display;

#[derive(Clone, Debug)]
pub enum TodoTypes {
    Done = 0,
    WorkInProgress = 1,
    Todo = 2,
}
impl TodoTypes {
    pub fn next(&self) -> Self {
        match self {
            TodoTypes::Done => TodoTypes::Todo,
            TodoTypes::WorkInProgress => TodoTypes::Done,
            TodoTypes::Todo => Self::WorkInProgress,
        }
    }
    pub fn prev(&self) -> Self {
        match self {
            TodoTypes::Done => TodoTypes::WorkInProgress,
            TodoTypes::WorkInProgress => TodoTypes::Todo,
            TodoTypes::Todo => Self::Done,
        }
    }
}

impl Display for TodoTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TodoTypes::Done => "Done",
                TodoTypes::WorkInProgress => "Work In progress",
                TodoTypes::Todo => "Todo",
            }
        );
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct Todo {
    pub todo_type: TodoTypes,
    pub text: String,
    pub children: Vec<Todo>,
}

impl Todo {
    pub(crate) fn new(text: String, todo_type: TodoTypes) -> Self {
        Self {
            todo_type: todo_type,
            text,
            children: vec![],
        }
    }
}
impl Display for Todo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{};{}", self.todo_type, self.text);
        Ok(())
    }
}
