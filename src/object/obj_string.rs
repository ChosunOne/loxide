use crate::object::Obj;
use std::fmt::Display;

#[derive(Debug, Default, PartialEq)]
pub struct ObjString {
    pub obj: Obj,
    pub hash: u32,
    pub chars: String,
}

impl Display for ObjString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.chars)
    }
}
