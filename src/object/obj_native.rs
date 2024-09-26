use crate::value::RuntimeValue;
use std::fmt::{Debug, Display};

pub type NativeFn = fn(Vec<RuntimeValue>) -> RuntimeValue;

#[derive(Clone, Copy, PartialEq)]
pub struct ObjNative {
    pub function: NativeFn,
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
