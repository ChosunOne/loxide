use std::{collections::HashMap, fmt::Display, hash::BuildHasherDefault};

use crate::{object::ObjClass, value::RuntimeValue};

use super::{HeapSize, ObjString, ObjStringHasher, Pointer};

#[derive(Clone, Debug, PartialEq)]
pub struct ObjInstance {
    pub class: Pointer<ObjClass>,
    pub fields: HashMap<ObjString, RuntimeValue, BuildHasherDefault<ObjStringHasher>>,
}

impl HeapSize for ObjInstance {
    fn size(&self) -> usize {
        size_of::<Pointer<ObjClass>>()
            + self.fields.keys().map(|x| x.chars.len()).sum::<usize>()
            + self.fields.values().map(size_of_val).sum::<usize>()
    }
}

impl Display for ObjInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class)
    }
}
