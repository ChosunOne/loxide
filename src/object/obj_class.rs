use crate::{object::ObjString, table::Table};
use std::fmt::Display;

use super::{HeapSize, ObjClosure, Pointer};

#[derive(Debug)]
pub struct ObjClass {
    pub name: Pointer<ObjString>,
    pub methods: Table<Pointer<ObjClosure>>,
}

impl PartialEq for ObjClass {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl HeapSize for ObjClass {
    fn size(&self) -> usize {
        size_of::<Pointer<ObjString>>() + self.methods.size()
    }
}

impl Display for ObjClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.borrow())
    }
}
