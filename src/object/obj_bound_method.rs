use crate::{
    object::ObjClosure,
    value::{RuntimeReference, RuntimeValue},
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ObjBoundMethod {
    pub receiver: RuntimeValue,
    pub method: RuntimeReference<ObjClosure>,
}
