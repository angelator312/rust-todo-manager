use std::fmt::Display;
use std::sync::atomic::{AtomicUsize, Ordering};

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
    pub children: Vec<usize>,
    pub parent: usize,
    id: usize,
}

fn get_id() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}
impl Todo {
    pub(crate) fn make_root() -> Self {
        Self {
            todo_type: TodoTypes::Done,
            text: "RootOfAll".into(),
            children: vec![],
            parent: 0,
            id: 0,
        }
    }
    pub(crate) fn new(text: String, todo_type: TodoTypes, parent: usize) -> Self {
        Self {
            todo_type,
            text,
            children: vec![],
            id: get_id(),
            parent,
        }
    }
    pub fn id(&self)->usize{
        self.id
    }
}
impl Display for Todo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{};{}", self.todo_type, self.text);
        Ok(())
    }
}
