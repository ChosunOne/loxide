use crate::value::RuntimeValue;
use std::fmt::Display;

use super::HeapSize;

#[derive(Clone, Debug, PartialEq)]
pub enum ObjUpvalue {
    /// The location **in the stack** where this variable's value can be found.
    Open { location: usize },
    /// The value that is no longer on the stack ("closed").
    Closed { value: RuntimeValue },
}

impl HeapSize for ObjUpvalue {
    fn size(&self) -> usize {
        match self {
            ObjUpvalue::Open { location } => size_of_val(location),
            ObjUpvalue::Closed { value } => size_of_val(value),
        }
    }
}

impl Display for ObjUpvalue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "upvalue")
    }
}
