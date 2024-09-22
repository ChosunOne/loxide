use crate::object::{ObjClosure, ObjString};
use std::collections::HashMap;

use super::Pointer;

#[derive(Debug, Clone, PartialEq)]
pub struct ObjClass {
    pub name: Pointer<ObjString>,
    pub methods: HashMap<String, Pointer<ObjClosure>>,
}
