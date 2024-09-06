use crate::{object::Obj, value::Value};
use std::{fmt::Display, rc::Rc};

#[derive(Debug, PartialEq)]
pub struct ObjUpvalue {
    pub obj: Obj,
    pub location: Rc<Value>,
    pub closed: Value,
    pub next: Option<Rc<ObjUpvalue>>,
}

impl Display for ObjUpvalue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "upvalue")
    }
}
