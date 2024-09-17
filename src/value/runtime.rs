use crate::{
    error::Error,
    object::{ObjBoundMethod, ObjClass, ObjClosure, ObjFunction, ObjInstance, ObjNative},
};

use super::{constant::ConstantValue, runtime_pointer::ObjectReference, RuntimeReference};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum RuntimeValue {
    Bool(bool),
    Number(f64),
    Object(ObjectReference),
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

impl From<ObjectReference> for RuntimeValue {
    fn from(value: ObjectReference) -> Self {
        Self::Object(value)
    }
}

impl From<RuntimeReference<ObjBoundMethod>> for RuntimeValue {
    fn from(value: RuntimeReference<ObjBoundMethod>) -> Self {
        Self::Object(value.into())
    }
}

impl From<RuntimeReference<ObjClass>> for RuntimeValue {
    fn from(value: RuntimeReference<ObjClass>) -> Self {
        Self::Object(value.into())
    }
}

impl From<RuntimeReference<ObjClosure>> for RuntimeValue {
    fn from(value: RuntimeReference<ObjClosure>) -> Self {
        Self::Object(value.into())
    }
}

impl From<RuntimeReference<ObjFunction>> for RuntimeValue {
    fn from(value: RuntimeReference<ObjFunction>) -> Self {
        Self::Object(value.into())
    }
}

impl From<RuntimeReference<ObjInstance>> for RuntimeValue {
    fn from(value: RuntimeReference<ObjInstance>) -> Self {
        Self::Object(value.into())
    }
}

impl From<RuntimeReference<ObjNative>> for RuntimeValue {
    fn from(value: RuntimeReference<ObjNative>) -> Self {
        Self::Object(value.into())
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
