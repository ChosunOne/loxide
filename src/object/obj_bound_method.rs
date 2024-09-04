use crate::{
    object::{Obj, ObjClosure},
    value::Value,
};

use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub struct ObjBoundMethod<'a> {
    pub obj: Obj<'a>,
    pub receiver: Value<'a>,
    pub method: &'a ObjClosure<'a>,
}

impl<'a> Display for ObjBoundMethod<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.method.function)
    }
}
