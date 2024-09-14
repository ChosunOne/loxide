use crate::error::Error;

use super::{constant::ConstantValue, runtime_pointer::ObjectReference};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum RuntimeValue {
    Bool(bool),
    Number(f64),
    Object(ObjectReference),
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

impl From<ObjectReference> for RuntimeValue {
    fn from(value: ObjectReference) -> Self {
        Self::Object(value)
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
