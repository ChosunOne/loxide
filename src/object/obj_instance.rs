use crate::object::{Obj, ObjClass};
use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub struct ObjInstance<'a> {
    pub obj: Obj<'a>,
    pub class: &'a ObjClass<'a>,
}

impl<'a> Display for ObjInstance<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class.name.chars)
    }
}
