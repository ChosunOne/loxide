use std::fmt::Display;

use super::HeapSize;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ObjString {
    pub chars: String,
}

impl HeapSize for ObjString {
    fn size(&self) -> usize {
        self.chars.len() + size_of::<String>()
    }
}

impl From<&str> for ObjString {
    fn from(value: &str) -> Self {
        Self {
            chars: value.into(),
        }
    }
}

impl From<String> for ObjString {
    fn from(value: String) -> Self {
        Self { chars: value }
    }
}

impl Display for ObjString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.chars)
    }
}
