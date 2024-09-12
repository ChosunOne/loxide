use std::fmt::Display;

use crate::object::{
    ObjBoundMethod, ObjClass, ObjClosure, ObjFunction, ObjInstance, ObjNative, ObjString,
    ObjUpvalue, Object,
};

use super::constant::ConstantValue;

#[derive(Clone, Debug, Default, PartialEq)]
pub enum RuntimeValue {
    Bool(bool),
    Number(f64),
    Object(Box<Object>),
    #[default]
    Nil,
}

impl From<bool> for RuntimeValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<f64> for RuntimeValue {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<&str> for RuntimeValue {
    fn from(value: &str) -> Self {
        Self::Object(Box::new(Object::String(value.into())))
    }
}

impl From<String> for RuntimeValue {
    fn from(value: String) -> Self {
        Self::Object(Box::new(Object::String(value.into())))
    }
}

impl From<ObjFunction> for RuntimeValue {
    fn from(value: ObjFunction) -> Self {
        Self::Object(Box::new(Object::Function(value)))
    }
}

impl From<ObjString> for RuntimeValue {
    fn from(value: ObjString) -> Self {
        Self::Object(Box::new(Object::String(value)))
    }
}

impl From<ObjClass> for RuntimeValue {
    fn from(value: ObjClass) -> Self {
        Self::Object(Box::new(Object::Class(value)))
    }
}

impl From<ObjBoundMethod> for RuntimeValue {
    fn from(value: ObjBoundMethod) -> Self {
        Self::Object(Box::new(Object::BoundMethod(value)))
    }
}

impl From<ObjInstance> for RuntimeValue {
    fn from(value: ObjInstance) -> Self {
        Self::Object(Box::new(Object::Instance(value)))
    }
}

impl From<ObjNative> for RuntimeValue {
    fn from(value: ObjNative) -> Self {
        Self::Object(Box::new(Object::Native(value)))
    }
}

impl From<ObjUpvalue> for RuntimeValue {
    fn from(value: ObjUpvalue) -> Self {
        Self::Object(Box::new(Object::UpValue(value)))
    }
}

impl From<ObjClosure> for RuntimeValue {
    fn from(value: ObjClosure) -> Self {
        Self::Object(Box::new(Object::Closure(value)))
    }
}

impl From<ConstantValue> for RuntimeValue {
    fn from(value: ConstantValue) -> Self {
        match value {
            ConstantValue::Number(n) => Self::Number(n),
            ConstantValue::String(s) => Self::Object(Box::new(Object::String(ObjString::from(s)))),
            ConstantValue::Function(f) => Self::Object(Box::new(Object::Function(*f))),
        }
    }
}

impl Display for RuntimeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(b) => write!(f, "{b}"),
            Self::Number(n) => write!(f, "{n}"),
            Self::Object(o) => write!(f, "{o}"),
            Self::Nil => write!(f, "nil"),
        }
    }
}
