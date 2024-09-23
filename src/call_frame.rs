use crate::object::{ObjClosure, Pointer};

#[derive(Debug, Clone, Default)]
pub struct CallFrame {
    /// A reference to the currently executing closure
    pub(crate) closure: Option<Pointer<ObjClosure>>,
    /// The index into the closure's code
    pub(crate) ip: usize,
    /// How many stack slots the call frame accesses
    pub(crate) slots: usize,
}
