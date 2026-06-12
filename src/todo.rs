use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::app::Id;

#[derive(Clone, Debug, Serialize, Deserialize)]
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
        )
    }
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Todo {
    pub todo_type: TodoTypes,
    pub text: String,
    pub id: Id,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Todo03 {
    pub todo_type: TodoTypes,
    pub text: String,
    pub children: Vec<Id>,
    pub parent: Id,
    pub id: Id,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TodoNode {
    pub children: Vec<Id>,
    pub parent: Id,
    pub id: Id,
}

impl TodoNode {
    pub(crate) fn make_root() -> Self {
        Self {
            children: vec![],
            parent: "0".into(),
            id: "0".into(),
        }
    }
    pub(crate) fn new(id: Id, parent: Id, children: Vec<Id>) -> TodoNode {
        Self {
            id,
            children,
            parent,
        }
    }
}

impl Todo {
    pub(crate) fn make_root() -> Self {
        Self {
            todo_type: TodoTypes::Done,
            text: "RootOfAll 1".into(),
            id: "0".into(),
        }
    }
    pub(crate) fn new(text: String, todo_type: TodoTypes, id: Id) -> Self {
        Self {
            todo_type,
            text,
            id,
        }
    }
}
impl Display for Todo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{};{}", self.todo_type, self.text)
    }
}
