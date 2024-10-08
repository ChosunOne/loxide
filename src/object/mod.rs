pub mod obj_bound_method;
pub mod obj_class;
pub mod obj_closure;
pub mod obj_function;
pub mod obj_instance;
pub mod obj_native;
pub mod obj_string;
pub mod obj_upvalue;
pub mod object_store;
pub mod store;

use std::hash::Hasher;

pub use obj_bound_method::ObjBoundMethod;
pub use obj_class::ObjClass;
pub use obj_closure::ObjClosure;
pub use obj_function::ObjFunction;
pub use obj_instance::ObjInstance;
pub use obj_native::ObjNative;
pub use obj_string::ObjString;
pub use obj_upvalue::ObjUpvalue;
pub use object_store::{ObjectStore, Pointer};
pub use store::Store;

pub trait HeapSize {
    /// The size of owned objects in the heap
    fn size(&self) -> usize;
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ObjStringHasher(u64);

impl Hasher for ObjStringHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, _bytes: &[u8]) {
        unreachable!("ObjStringHasher::write should not be called");
    }

    fn write_u32(&mut self, i: u32) {
        self.0 = i as u64;
    }
}
