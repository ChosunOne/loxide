use crate::{
    chunk::Chunk,
    object::{Obj, ObjString},
};
use std::{fmt::Display, rc::Rc};

#[derive(Debug, Default, PartialEq)]
pub struct ObjFunction {
    pub obj: Obj,
    pub arity: usize,
    pub upvalue_count: usize,
    pub chunk: Chunk,
    pub name: Option<Rc<ObjString>>,
}

impl Display for ObjFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.name.is_none() {
            return write!(f, "<script>");
        }
        write!(f, "<fn {}>", self.name.as_ref().unwrap().chars)
    }
}
