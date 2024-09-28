use std::fmt::Display;

use crate::{object::ObjClosure, value::RuntimeValue};

use super::{HeapSize, Pointer};

#[derive(Debug, Clone, PartialEq)]
pub struct ObjBoundMethod {
    pub receiver: RuntimeValue,
    pub method: Pointer<ObjClosure>,
}

impl HeapSize for ObjBoundMethod {
    fn size(&self) -> usize {
        size_of_val(self)
    }
}

impl Display for ObjBoundMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.method)
    }
}
