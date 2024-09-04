use crate::object::{Obj, ObjFunction, ObjString};
use std::{collections::HashMap, fmt::Display};

#[derive(Debug, PartialEq)]
pub struct ObjClass<'a> {
    pub obj: Obj<'a>,
    pub name: &'a ObjString<'a>,
    pub methods: HashMap<String, &'a ObjFunction<'a>>,
}

impl<'a> Display for ObjClass<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.chars)
    }
}
