use crate::object::{Obj, ObjFunction, ObjUpvalue};
use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub struct ObjClosure<'a> {
    pub obj: Obj<'a>,
    pub function: &'a ObjFunction<'a>,
    pub upvalues: Vec<&'a ObjUpvalue<'a>>,
}

impl<'a> Display for ObjClosure<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.function)
    }
}
