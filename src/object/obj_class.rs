use crate::{
    object::{ObjClosure, ObjString},
    value::RuntimeReference,
};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct ObjClass {
    pub name: RuntimeReference<ObjString>,
    pub methods: HashMap<String, RuntimeReference<ObjClosure>>,
}
