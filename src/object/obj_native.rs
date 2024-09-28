use crate::value::RuntimeValue;
use std::fmt::{Debug, Display};

use super::HeapSize;

pub type NativeFn = fn(Vec<RuntimeValue>) -> RuntimeValue;

#[derive(Clone, Copy, PartialEq)]
pub struct ObjNative {
    pub function: NativeFn,
}

impl HeapSize for ObjNative {
    fn size(&self) -> usize {
        size_of_val(self)
    }
}

impl Debug for ObjNative {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ObjNative {{ function: <native fn>}}",)
    }
}

impl Display for ObjNative {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn>")
    }
}
