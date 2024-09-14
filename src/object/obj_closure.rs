use crate::{
    object::{ObjFunction, ObjUpvalue},
    value::RuntimeReference,
};

#[derive(Clone, Debug, PartialEq)]
pub struct ObjClosure {
    pub function: RuntimeReference<ObjFunction>,
    pub upvalues: Vec<RuntimeReference<ObjUpvalue>>,
}
