use crate::object::{ObjClosure, ObjString};
use std::{collections::HashMap, fmt::Display};

use super::Pointer;

#[derive(Debug, Clone, PartialEq)]
pub struct ObjClass {
    pub name: Pointer<ObjString>,
    pub methods: HashMap<String, Pointer<ObjClosure>>,
}

impl Display for ObjClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.borrow())
    }
}
