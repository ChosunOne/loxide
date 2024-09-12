use crate::value::RuntimeValue;
use std::{fmt::Display, rc::Rc};

#[derive(Clone, Debug, PartialEq)]
pub struct ObjUpvalue {
    pub location: Rc<RuntimeValue>,
    pub closed: RuntimeValue,
    pub next: Option<Rc<ObjUpvalue>>,
}

impl Display for ObjUpvalue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "upvalue")
    }
}
