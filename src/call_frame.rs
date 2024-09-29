use std::{cell::RefCell, rc::Rc};

use crate::{
    chunk::Chunk,
    object::{ObjClosure, Pointer},
};

#[derive(Debug, Clone, Default)]
pub struct CallFrame {
    /// A reference to the currently executing function
    pub(crate) chunk: Rc<RefCell<Chunk>>,
    /// A reference to the currently executing closure
    pub(crate) closure: Option<Pointer<ObjClosure>>,
    /// The index into the closure's code
    pub(crate) ip: usize,
    /// How many stack slots the call frame accesses
    pub(crate) slots: usize,
    /// The absolute index of the start of the call frame
    pub(crate) start_stack_index: usize,
}
