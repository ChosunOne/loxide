use std::{collections::HashMap, fmt::Display};

use crate::{object::ObjClass, value::RuntimeValue};

use super::Pointer;

#[derive(Clone, Debug, PartialEq)]
pub struct ObjInstance {
    pub class: Pointer<ObjClass>,
    pub fields: HashMap<String, RuntimeValue>,
}

impl Display for ObjInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class)
    }
}
