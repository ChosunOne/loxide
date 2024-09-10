use crate::object::Obj;
use std::fmt::Display;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ObjString {
    pub obj: Obj,
    pub chars: String,
}

impl From<&str> for ObjString {
    fn from(value: &str) -> Self {
        Self {
            obj: Obj::default(),
            chars: value.into(),
        }
    }
}

impl From<String> for ObjString {
    fn from(value: String) -> Self {
        Self {
            obj: Obj::default(),
            chars: value,
        }
    }
}

impl Display for ObjString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.chars)
    }
}
