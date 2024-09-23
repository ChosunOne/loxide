use std::fmt::Display;

use crate::{
    error::Error,
    object::{
        ObjBoundMethod, ObjClass, ObjClosure, ObjFunction, ObjInstance, ObjNative, ObjString,
        Pointer,
    },
};

use super::constant::ConstantValue;

#[derive(Clone, Debug, Default, PartialEq)]
pub enum RuntimeValue {
    Bool(bool),
    Number(f64),
    BoundMethod(Pointer<ObjBoundMethod>),
    Class(Pointer<ObjClass>),
    Closure(Pointer<ObjClosure>),
    Function(Pointer<ObjFunction>),
    Instance(Pointer<ObjInstance>),
    Native(Pointer<ObjNative>),
    String(Pointer<ObjString>),
    #[default]
    Nil,
}

impl RuntimeValue {
    pub fn is_falsey(&self) -> bool {
        match self {
            Self::Nil => true,
            Self::Bool(b) => !b,
            _ => false,
        }
    }
}

impl Display for RuntimeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeValue::Bool(b) => write!(f, "{b}"),
            RuntimeValue::Number(n) => write!(f, "{n}"),
            RuntimeValue::BoundMethod(pointer) => write!(f, "{pointer}"),
            RuntimeValue::Class(pointer) => write!(f, "{pointer}"),
            RuntimeValue::Closure(pointer) => write!(f, "{pointer}"),
            RuntimeValue::Function(pointer) => write!(f, "{pointer}"),
            RuntimeValue::Instance(pointer) => write!(f, "{pointer}"),
            RuntimeValue::Native(pointer) => write!(f, "{pointer}"),
            RuntimeValue::String(pointer) => write!(f, "{pointer}"),
            RuntimeValue::Nil => write!(f, "nil"),
        }
    }
}

impl TryFrom<RuntimeValue> for usize {
    type Error = Error;

    fn try_from(value: RuntimeValue) -> Result<Self, Self::Error> {
        match value {
            RuntimeValue::Number(n) => Ok(n as usize),
            _ => Err(Error::Runtime),
        }
    }
}

impl TryFrom<RuntimeValue> for f64 {
    type Error = Error;

    fn try_from(value: RuntimeValue) -> Result<Self, Self::Error> {
        match value {
            RuntimeValue::Number(n) => Ok(n),
            _ => Err(Error::Runtime),
        }
    }
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

impl From<usize> for RuntimeValue {
    fn from(value: usize) -> Self {
        Self::Number(value as f64)
    }
}

impl From<Pointer<ObjBoundMethod>> for RuntimeValue {
    fn from(value: Pointer<ObjBoundMethod>) -> Self {
        Self::BoundMethod(value)
    }
}

impl From<Pointer<ObjClass>> for RuntimeValue {
    fn from(value: Pointer<ObjClass>) -> Self {
        Self::Class(value)
    }
}

impl From<Pointer<ObjClosure>> for RuntimeValue {
    fn from(value: Pointer<ObjClosure>) -> Self {
        Self::Closure(value)
    }
}

impl From<Pointer<ObjFunction>> for RuntimeValue {
    fn from(value: Pointer<ObjFunction>) -> Self {
        Self::Function(value)
    }
}

impl From<Pointer<ObjInstance>> for RuntimeValue {
    fn from(value: Pointer<ObjInstance>) -> Self {
        Self::Instance(value)
    }
}

impl From<Pointer<ObjNative>> for RuntimeValue {
    fn from(value: Pointer<ObjNative>) -> Self {
        Self::Native(value)
    }
}

impl From<Pointer<ObjString>> for RuntimeValue {
    fn from(value: Pointer<ObjString>) -> Self {
        Self::String(value)
    }
}

impl TryFrom<ConstantValue> for RuntimeValue {
    type Error = Error;

    fn try_from(value: ConstantValue) -> Result<Self, Error> {
        match value {
            ConstantValue::Number(n) => Ok(Self::Number(n)),
            _ => Err(Error::Runtime),
        }
    }
}
