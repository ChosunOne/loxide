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

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::Object(Box::new(Object::String(value.into())))
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::Object(Box::new(Object::String(value.into())))
    }
}

impl From<ObjFunction> for Value {
    fn from(value: ObjFunction) -> Self {
        Self::Object(Box::new(Object::Function(value)))
    }
}

impl From<ObjString> for Value {
    fn from(value: ObjString) -> Self {
        Self::Object(Box::new(Object::String(value)))
    }
}

impl From<ObjClass> for Value {
    fn from(value: ObjClass) -> Self {
        Self::Object(Box::new(Object::Class(value)))
    }
}

impl From<ObjBoundMethod> for Value {
    fn from(value: ObjBoundMethod) -> Self {
        Self::Object(Box::new(Object::BoundMethod(value)))
    }
}

impl From<ObjInstance> for Value {
    fn from(value: ObjInstance) -> Self {
        Self::Object(Box::new(Object::Instance(value)))
    }
}

impl From<ObjNative> for Value {
    fn from(value: ObjNative) -> Self {
        Self::Object(Box::new(Object::Native(value)))
    }
}

impl From<ObjUpvalue> for Value {
    fn from(value: ObjUpvalue) -> Self {
        Self::Object(Box::new(Object::UpValue(value)))
    }
}

impl From<ObjClosure> for Value {
    fn from(value: ObjClosure) -> Self {
        Self::Object(Box::new(Object::Closure(value)))
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
