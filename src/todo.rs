use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::app::Id;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Todo {
    pub todo_type: String,
    pub text: String,
    pub id: Id,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Todo03 {
    pub todo_type: String,
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
            todo_type: "Todo: Done".into(),
            text: "RootOfAll 1".into(),
            id: "0".into(),
        }
    }
    pub(crate) fn new(text: String, todo_type: String, id: Id) -> Self {
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
