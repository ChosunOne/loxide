use crate::object::{Obj, ObjClass};
use std::{fmt::Display, rc::Rc};

#[derive(Clone, Debug, PartialEq)]
pub struct ObjInstance {
    pub obj: Obj,
    pub class: Rc<ObjClass>,
}

impl Display for ObjInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class.name.chars)
    }
}
