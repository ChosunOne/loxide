use crate::{object::Obj, value::Value};
use std::fmt::{Debug, Display};

type NativeFn = Box<dyn Fn(Vec<Value<'static>>) -> Value<'static>>;

pub struct ObjNative<'a> {
    pub obj: Obj<'a>,
    pub function: NativeFn,
}

impl<'a> PartialEq for ObjNative<'a> {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl<'a> Debug for ObjNative<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ObjNative {{ obj: {:?}, function: <native fn>}}",
            self.obj
        )
    }
}

impl<'a> Display for ObjNative<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn>")
    }
}
