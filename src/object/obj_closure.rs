use std::fmt::Display;

use crate::object::{ObjFunction, ObjUpvalue, Pointer};

#[derive(Clone, Debug, PartialEq)]
pub struct ObjClosure {
    pub function: Pointer<ObjFunction>,
    pub upvalues: Vec<Pointer<ObjUpvalue>>,
}

impl Display for ObjClosure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self.function)
    }
}
