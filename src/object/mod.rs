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
    Closure(ObjClosure),
    Function(ObjFunction),
    Instance(ObjInstance),
    Native(ObjNative),
    String(ObjString),
    UpValue(ObjUpvalue),
}

impl From<ObjBoundMethod> for Object {
    fn from(value: ObjBoundMethod) -> Self {
        Self::BoundMethod(value)
    }
}

impl From<ObjClass> for Object {
    fn from(value: ObjClass) -> Self {
        Self::Class(value)
    }
}

impl From<ObjClosure> for Object {
    fn from(value: ObjClosure) -> Self {
        Self::Closure(value)
    }
}

impl From<ObjFunction> for Object {
    fn from(value: ObjFunction) -> Self {
        Self::Function(value)
    }
}

impl From<ObjInstance> for Object {
    fn from(value: ObjInstance) -> Self {
        Self::Instance(value)
    }
}

impl From<ObjNative> for Object {
    fn from(value: ObjNative) -> Self {
        Self::Native(value)
    }
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
