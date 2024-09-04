use crate::{object::Obj, value::Value};
use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub struct ObjUpvalue<'a> {
    pub obj: Obj<'a>,
    pub location: &'a Value<'a>,
    pub closed: Value<'a>,
    pub next: Option<&'a ObjUpvalue<'a>>,
}

impl<'a> Display for ObjUpvalue<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "upvalue")
    }
}
