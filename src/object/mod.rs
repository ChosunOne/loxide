pub mod obj_bound_method;
pub mod obj_class;
pub mod obj_closure;
pub mod obj_function;
pub mod obj_instance;
pub mod obj_native;
pub mod obj_string;
pub mod obj_upvalue;
pub mod object_store;

pub use obj_bound_method::ObjBoundMethod;
pub use obj_class::ObjClass;
pub use obj_closure::ObjClosure;
pub use obj_function::ObjFunction;
pub use obj_instance::ObjInstance;
pub use obj_native::ObjNative;
pub use obj_string::ObjString;
pub use obj_upvalue::ObjUpvalue;
pub use object_store::ObjectStore;

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    BoundMethod(ObjBoundMethod),
    Class(ObjClass),
    Function(ObjFunction),
    Instance(ObjInstance),
    Native(ObjNative),
    String(ObjString),
    UpValue(ObjUpvalue),
    Closure(ObjClosure),
}

impl From<&str> for Object {
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}

impl From<String> for Object {
    fn from(value: String) -> Self {
        Self::String(value.into())
    }
}
