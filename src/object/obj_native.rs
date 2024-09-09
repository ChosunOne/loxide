use crate::{object::Obj, value::Value};
use std::fmt::{Debug, Display};

type NativeFn = fn(Vec<Value>) -> Value;

#[derive(Clone, PartialEq)]
pub struct ObjNative {
    pub obj: Obj,
    pub function: NativeFn,
}

impl Debug for ObjNative {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ObjNative {{ obj: {:?}, function: <native fn>}}",
            self.obj
        )
    }
}

impl Display for ObjNative {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn>")
    }
}
