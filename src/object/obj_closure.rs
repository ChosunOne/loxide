use crate::object::{Obj, ObjFunction, ObjUpvalue};
use std::{fmt::Display, rc::Rc};

#[derive(Debug, PartialEq)]
pub struct ObjClosure {
    pub obj: Obj,
    pub function: Rc<ObjFunction>,
    pub upvalues: Vec<Rc<ObjUpvalue>>,
}

impl Display for ObjClosure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.function)
    }
}
