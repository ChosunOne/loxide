use crate::object::{Obj, ObjFunction, ObjString};
use std::{collections::HashMap, fmt::Display, rc::Rc};

#[derive(Debug, Clone, PartialEq)]
pub struct ObjClass {
    pub obj: Obj,
    pub name: Rc<ObjString>,
    pub methods: HashMap<String, Rc<ObjFunction>>,
}

impl Display for ObjClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.chars)
    }
}
