use std::collections::HashMap;

use crate::{object::ObjClass, value::RuntimeValue};

use super::Pointer;

#[derive(Clone, Debug, PartialEq)]
pub struct ObjInstance {
    pub class: Pointer<ObjClass>,
    pub fields: HashMap<String, RuntimeValue>,
}
