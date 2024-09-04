use crate::object::Obj;
use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub struct ObjString<'a> {
    pub obj: Obj<'a>,
    pub hash: u32,
    pub chars: String,
}

impl<'a> Display for ObjString<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.chars)
    }
}
