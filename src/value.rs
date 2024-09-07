use std::fmt::Display;

use crate::object::{
    ObjBoundMethod, ObjClass, ObjClosure, ObjFunction, ObjInstance, ObjNative, ObjString,
    ObjUpvalue, Object,
};

#[derive(Debug, Default, PartialEq)]
pub enum Value {
    Bool(bool),
    Number(f64),
    Object(Box<Object>),
    #[default]
    Nil,
}

impl Value {
    pub fn new_function(function: ObjFunction) -> Self {
        Self::Object(Box::new(Object::Function(function)))
    }

    pub fn new_string(string: ObjString) -> Self {
        Self::Object(Box::new(Object::String(string)))
    }

    pub fn new_class(class: ObjClass) -> Self {
        Self::Object(Box::new(Object::Class(class)))
    }

    pub fn new_bound_method(bound_method: ObjBoundMethod) -> Self {
        Self::Object(Box::new(Object::BoundMethod(bound_method)))
    }

    pub fn new_instance(instance: ObjInstance) -> Self {
        Self::Object(Box::new(Object::Instance(instance)))
    }

    pub fn new_native(native: ObjNative) -> Self {
        Self::Object(Box::new(Object::Native(native)))
    }

    pub fn new_upvalue(upvalue: ObjUpvalue) -> Self {
        Self::Object(Box::new(Object::UpValue(upvalue)))
    }

    pub fn new_closure(closure: ObjClosure) -> Self {
        Self::Object(Box::new(Object::Closure(closure)))
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(b) => write!(f, "{b}"),
            Self::Number(n) => write!(f, "{n}"),
            Self::Object(o) => write!(f, "{o}"),
            Self::Nil => write!(f, "nil"),
        }
    }
}
