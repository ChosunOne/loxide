use std::ptr::null;

use crate::{
    chunk::Chunk,
    object::{ObjClosure, Pointer},
};

#[derive(Debug, Clone)]
pub struct CallFrame {
    /// A reference to the currently executing function
    pub(crate) chunk: *const Chunk,
    /// A reference to the currently executing closure
    pub(crate) closure: Pointer<ObjClosure>,
    /// The index into the closure's code
    pub(crate) ip: usize,
    /// How many stack slots the call frame accesses
    pub(crate) slots: usize,
    /// The absolute index of the start of the call frame
    pub(crate) start_stack_index: usize,
}

impl Default for CallFrame {
    fn default() -> Self {
        Self {
            chunk: null(),
            closure: Pointer::default(),
            ip: 0,
            slots: 0,
            start_stack_index: 0,
        }
    }
}
