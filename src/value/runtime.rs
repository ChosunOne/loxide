use std::{fmt::Display, hash::Hash};

use crate::{
    error::Error,
    object::{
        HeapSize, ObjBoundMethod, ObjClass, ObjClosure, ObjFunction, ObjInstance, ObjNative,
        ObjString, ObjUpvalue, Pointer,
    },
};

use super::constant::ConstantValue;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
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
    Upvalue(Pointer<ObjUpvalue>),
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

impl HeapSize for RuntimeValue {
    fn size(&self) -> usize {
        size_of::<RuntimeValue>()
    }
}

impl Hash for RuntimeValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            RuntimeValue::Bool(b) => b.hash(state),
            RuntimeValue::Number(n) => n.to_bits().hash(state),
            RuntimeValue::BoundMethod(pointer) => pointer.hash(state),
            RuntimeValue::Class(pointer) => pointer.hash(state),
            RuntimeValue::Closure(pointer) => pointer.hash(state),
            RuntimeValue::Function(pointer) => pointer.hash(state),
            RuntimeValue::Instance(pointer) => pointer.hash(state),
            RuntimeValue::Native(pointer) => pointer.hash(state),
            RuntimeValue::String(pointer) => pointer.hash(state),
            RuntimeValue::Upvalue(pointer) => pointer.hash(state),
            RuntimeValue::Nil => 0.hash(state),
        }
    }
}

impl Eq for RuntimeValue {}

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
            RuntimeValue::Upvalue(pointer) => write!(f, "{pointer}"),
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

impl From<Pointer<ObjUpvalue>> for RuntimeValue {
    fn from(value: Pointer<ObjUpvalue>) -> Self {
        Self::Upvalue(value)
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
