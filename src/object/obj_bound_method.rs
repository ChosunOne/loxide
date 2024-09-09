use crate::{
    object::{Obj, ObjClosure},
    value::Value,
};

use std::{fmt::Display, rc::Rc};

#[derive(Debug, Clone, PartialEq)]
pub struct ObjBoundMethod {
    pub obj: Obj,
    pub receiver: Value,
    pub method: Rc<ObjClosure>,
}

impl Display for ObjBoundMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.method.function)
    }
}
