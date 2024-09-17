use crate::{object::ObjClosure, value::RuntimeReference};

#[derive(Debug, Clone, Copy)]
pub struct CallFrame {
    /// A reference to the currently executing closure
    pub(crate) closure: RuntimeReference<ObjClosure>,
    /// The index into the closure's code
    pub(crate) ip: usize,
    /// How many stack slots the call frame accesses
    pub(crate) slots: usize,
}
