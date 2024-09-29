use crate::object::{ObjClosure, ObjString};
use std::{collections::HashMap, fmt::Display, hash::BuildHasherDefault};

use super::{HeapSize, ObjStringHasher, Pointer};

#[derive(Debug, Clone, PartialEq)]
pub struct ObjClass {
    pub name: Pointer<ObjString>,
    pub methods: HashMap<ObjString, Pointer<ObjClosure>, BuildHasherDefault<ObjStringHasher>>,
}

impl HeapSize for ObjClass {
    fn size(&self) -> usize {
        self.methods.len() * size_of::<Pointer<ObjClosure>>()
            + self.methods.keys().map(|x| x.chars.len()).sum::<usize>()
    }
}

impl Display for ObjClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.borrow())
    }
}
