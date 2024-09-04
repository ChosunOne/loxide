use crate::{
    chunk::Chunk,
    object::{Obj, ObjString},
};
use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub struct ObjFunction<'a> {
    pub obj: Obj<'a>,
    pub arity: usize,
    pub upvalue_count: usize,
    pub chunk: Chunk<'a>,
    pub name: Option<&'a ObjString<'a>>,
}

impl<'a> Display for ObjFunction<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.name.is_none() {
            return write!(f, "<script>");
        }
        write!(f, "<fn {}>", self.name.unwrap().chars)
    }
}
