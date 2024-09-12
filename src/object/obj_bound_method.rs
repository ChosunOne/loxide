use crate::{object::ObjClosure, value::RuntimeValue};

use std::{fmt::Display, rc::Rc};

#[derive(Debug, Clone, PartialEq)]
pub struct ObjBoundMethod {
    pub receiver: RuntimeValue,
    pub method: Rc<ObjClosure>,
}

impl Display for ObjBoundMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.method.function)
    }
}
