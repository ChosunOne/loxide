use std::fmt::Display;

use crate::{object::ObjClass, table::Table};

use super::{HeapSize, Pointer};

#[derive(Debug)]
pub struct ObjInstance {
    pub class: Pointer<ObjClass>,
    pub fields: Table,
}

impl PartialEq for ObjInstance {
    fn eq(&self, other: &Self) -> bool {
        self.class == other.class
    }
}

impl HeapSize for ObjInstance {
    fn size(&self) -> usize {
        size_of::<Pointer<ObjClass>>() + self.fields.size()
    }
}

impl Display for ObjInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class)
    }
}
