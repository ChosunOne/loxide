use std::fmt::Display;

use crate::object::Object;

#[derive(Debug, Default, PartialEq)]
pub enum Value {
    Bool(bool),
    Number(f64),
    Object(Box<Object>),
    #[default]
    Nil,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(b) => write!(f, "{b}"),
            Self::Number(n) => write!(f, "{n}"),
            Self::Object(o) => write!(f, "{o}"),
            Self::Nil => write!(f, "nil"),
        }
    }
}
