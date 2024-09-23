use std::fmt::Display;

use crate::{object::ObjClosure, value::RuntimeValue};

use super::Pointer;

#[derive(Debug, Clone, PartialEq)]
pub struct ObjBoundMethod {
    pub receiver: RuntimeValue,
    pub method: Pointer<ObjClosure>,
}

impl Display for ObjBoundMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.method)
    }
}
