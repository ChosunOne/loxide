use crate::object::{ObjFunction, ObjUpvalue};
use std::{fmt::Display, rc::Rc};

#[derive(Clone, Debug, PartialEq)]
pub struct ObjClosure {
    pub function: Rc<ObjFunction>,
    pub upvalues: Vec<Rc<ObjUpvalue>>,
}

impl Display for ObjClosure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.function)
    }
}
