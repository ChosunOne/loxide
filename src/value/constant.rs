use std::fmt::Display;

use crate::{error::Error, object::ObjFunction, value::RuntimeValue};

#[derive(Debug, Clone, PartialEq)]
pub enum ConstantValue {
    Number(f64),
    String(String),
    Function(Box<ObjFunction>),
}

impl From<f64> for ConstantValue {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<String> for ConstantValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for ConstantValue {
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}

impl From<ObjFunction> for ConstantValue {
    fn from(value: ObjFunction) -> Self {
        Self::Function(Box::new(value))
    }
}

impl TryFrom<RuntimeValue> for ConstantValue {
    type Error = Error;

    fn try_from(value: RuntimeValue) -> Result<Self, Self::Error> {
        match value {
            RuntimeValue::Number(n) => Ok(Self::Number(n)),
            _ => Err(Error::Runtime),
        }
    }
}

impl Display for ConstantValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{n}"),
            Self::String(s) => write!(f, "{s}"),
            Self::Function(fun) => write!(f, "{fun}"),
        }
    }
}
