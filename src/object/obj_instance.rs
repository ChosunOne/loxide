use std::collections::HashMap;

use crate::{
    object::ObjClass,
    value::{RuntimeReference, RuntimeValue},
};

#[derive(Clone, Debug, PartialEq)]
pub struct ObjInstance {
    pub class: RuntimeReference<ObjClass>,
    pub fields: HashMap<String, RuntimeValue>,
}
