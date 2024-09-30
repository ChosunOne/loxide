use std::fmt::Display;

use crate::object::{ObjFunction, ObjUpvalue, Pointer};

use super::HeapSize;

#[derive(Clone, Debug, PartialEq)]
pub struct ObjClosure {
    pub function: Pointer<ObjFunction>,
    pub upvalues: Vec<Pointer<ObjUpvalue>>,
}

impl HeapSize for ObjClosure {
    fn size(&self) -> usize {
        size_of_val(self)
    }
}

impl Display for ObjClosure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.function)
    }
}
