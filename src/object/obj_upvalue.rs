use crate::value::{RuntimeReference, RuntimeValue};
use std::fmt::Display;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ObjUpvalue {
    pub location: RuntimeReference<RuntimeValue>,
    pub closed: RuntimeValue,
    pub next: Option<RuntimeReference<ObjUpvalue>>,
}

impl Display for ObjUpvalue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "upvalue")
    }
}
