use crate::{
    object::ObjClosure,
    value::{RuntimeReference, RuntimeValue},
};

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub(crate) closure: RuntimeReference<ObjClosure>,
    pub(crate) ip: u8,
    pub(crate) slots: Vec<RuntimeValue>,
}
