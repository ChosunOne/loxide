use crate::{object::ObjClosure, value::RuntimeValue};

use super::Pointer;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ObjBoundMethod {
    pub receiver: RuntimeValue,
    pub method: Pointer<ObjClosure>,
}
