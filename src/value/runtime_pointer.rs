use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::{
    error::Error,
    object::{
        ObjBoundMethod, ObjClass, ObjClosure, ObjFunction, ObjInstance, ObjNative, ObjString,
        ObjUpvalue, Object, ObjectStore,
    },
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ObjectReference {
    BoundMethod(RuntimeReference<ObjBoundMethod>),
    Class(RuntimeReference<ObjClass>),
    Closure(RuntimeReference<ObjClosure>),
    Function(RuntimeReference<ObjFunction>),
    Instance(RuntimeReference<ObjInstance>),
    Native(RuntimeReference<ObjNative>),
    String(RuntimeReference<ObjString>),
    Upvalue(RuntimeReference<ObjUpvalue>),
}

impl From<RuntimeReference<ObjBoundMethod>> for ObjectReference {
    fn from(value: RuntimeReference<ObjBoundMethod>) -> Self {
        Self::BoundMethod(value)
    }
}

impl From<RuntimeReference<ObjClass>> for ObjectReference {
    fn from(value: RuntimeReference<ObjClass>) -> Self {
        Self::Class(value)
    }
}

impl From<RuntimeReference<ObjClosure>> for ObjectReference {
    fn from(value: RuntimeReference<ObjClosure>) -> Self {
        Self::Closure(value)
    }
}

impl From<RuntimeReference<ObjFunction>> for ObjectReference {
    fn from(value: RuntimeReference<ObjFunction>) -> Self {
        Self::Function(value)
    }
}

impl From<RuntimeReference<ObjInstance>> for ObjectReference {
    fn from(value: RuntimeReference<ObjInstance>) -> Self {
        Self::Instance(value)
    }
}

impl From<RuntimeReference<ObjNative>> for ObjectReference {
    fn from(value: RuntimeReference<ObjNative>) -> Self {
        Self::Native(value)
    }
}

impl From<RuntimeReference<ObjString>> for ObjectReference {
    fn from(value: RuntimeReference<ObjString>) -> Self {
        Self::String(value)
    }
}

impl From<RuntimeReference<ObjUpvalue>> for ObjectReference {
    fn from(value: RuntimeReference<ObjUpvalue>) -> Self {
        Self::Upvalue(value)
    }
}

#[derive(Debug, PartialEq)]
pub struct RuntimeReference<T> {
    pub(crate) object_ref: *const Object,
    pub(crate) _phantom: PhantomData<T>,
}

impl<T> Clone for RuntimeReference<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for RuntimeReference<T> {}

impl<T> From<&Object> for RuntimeReference<T> {
    fn from(value: &Object) -> Self {
        Self {
            object_ref: value as *const Object,
            _phantom: PhantomData,
        }
    }
}

impl<T> From<&RuntimePointer<'_, T>> for RuntimeReference<T> {
    fn from(value: &RuntimePointer<'_, T>) -> Self {
        Self {
            object_ref: value.object_ref,
            _phantom: PhantomData,
        }
    }
}

impl TryFrom<ObjectReference> for RuntimeReference<ObjBoundMethod> {
    type Error = Error;

    fn try_from(value: ObjectReference) -> Result<Self, Error> {
        match value {
            ObjectReference::BoundMethod(bm) => Ok(bm),
            _ => Err(Error::Runtime),
        }
    }
}

impl TryFrom<ObjectReference> for RuntimeReference<ObjClass> {
    type Error = Error;

    fn try_from(value: ObjectReference) -> Result<Self, Error> {
        match value {
            ObjectReference::Class(c) => Ok(c),
            _ => Err(Error::Runtime),
        }
    }
}

impl TryFrom<ObjectReference> for RuntimeReference<ObjClosure> {
    type Error = Error;

    fn try_from(value: ObjectReference) -> Result<Self, Error> {
        match value {
            ObjectReference::Closure(c) => Ok(c),
            _ => Err(Error::Runtime),
        }
    }
}

impl TryFrom<ObjectReference> for RuntimeReference<ObjFunction> {
    type Error = Error;

    fn try_from(value: ObjectReference) -> Result<Self, Error> {
        match value {
            ObjectReference::Function(f) => Ok(f),
            _ => Err(Error::Runtime),
        }
    }
}

impl TryFrom<ObjectReference> for RuntimeReference<ObjInstance> {
    type Error = Error;

    fn try_from(value: ObjectReference) -> Result<Self, Error> {
        match value {
            ObjectReference::Instance(i) => Ok(i),
            _ => Err(Error::Runtime),
        }
    }
}

impl TryFrom<ObjectReference> for RuntimeReference<ObjNative> {
    type Error = Error;

    fn try_from(value: ObjectReference) -> Result<Self, Error> {
        match value {
            ObjectReference::Native(n) => Ok(n),
            _ => Err(Error::Runtime),
        }
    }
}

impl TryFrom<ObjectReference> for RuntimeReference<ObjString> {
    type Error = Error;

    fn try_from(value: ObjectReference) -> Result<Self, Error> {
        match value {
            ObjectReference::String(s) => Ok(s),
            _ => Err(Error::Runtime),
        }
    }
}

impl TryFrom<ObjectReference> for RuntimeReference<ObjUpvalue> {
    type Error = Error;

    fn try_from(value: ObjectReference) -> Result<Self, Error> {
        match value {
            ObjectReference::Upvalue(u) => Ok(u),
            _ => Err(Error::Runtime),
        }
    }
}

#[derive(Debug)]
pub struct RuntimePointer<'a, T> {
    pub(crate) object_store: &'a mut ObjectStore,
    pub(crate) object_ref: *const Object,
    pub(crate) _phantom: PhantomData<T>,
}

impl<'a, T> RuntimePointer<'a, T> {
    pub fn as_raw_ptr(self) -> *const Object {
        self.object_ref
    }
}

macro_rules! impl_runtime_pointer_deref {
    ($type:ty, $variant:pat => $result:expr) => {
        impl<'a> Deref for RuntimePointer<'a, $type> {
            type Target = $type;

            fn deref(&self) -> &Self::Target {
                match self
                    .object_store
                    .get(self.object_ref)
                    .expect("Failed to get object from store.")
                {
                    $variant => $result,
                    o => panic!("Unexpected object: {o:?}"),
                }
            }
        }
    };
    ($type:ty, $target:ty, $variant:pat => $result:expr) => {
        impl<'a> Deref for RuntimePointer<'a, $type> {
            type Target = $target;

            fn deref(&self) -> &Self::Target {
                match self
                    .object_store
                    .get(self.object_ref)
                    .expect("Failed to get object from store.")
                {
                    $variant => $result,
                    o => panic!("Unexpected object: {o:?}"),
                }
            }
        }
    };
    ($type:ty, $variant:pat => $result:expr, mut) => {
        impl<'a> Deref for RuntimePointer<'a, $type> {
            type Target = $type;

            fn deref(&self) -> &Self::Target {
                match self
                    .object_store
                    .get(self.object_ref)
                    .expect("Failed to get object from store.")
                {
                    $variant => $result,
                    o => panic!("Unexpected object: {o:?}"),
                }
            }
        }

        impl<'a> DerefMut for RuntimePointer<'a, $type> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                match self
                    .object_store
                    .get_mut(self.object_ref)
                    .expect("Failed to get object from store.")
                {
                    $variant => $result,
                    o => panic!("Unexpected object: {o:?}"),
                }
            }
        }
    };
    ($type:ty, $variant:pat => $result:expr, $variant_mut:pat => $result_mut:expr) => {
        impl<'a> Deref for RuntimePointer<'a, $type> {
            type Target = $type;

            fn deref(&self) -> &Self::Target {
                match self
                    .object_store
                    .get(self.object_ref)
                    .expect("Failed to get object from store.")
                {
                    $variant => $result,
                    o => panic!("Unexpected object: {o:?}"),
                }
            }
        }

        impl<'a> DerefMut for RuntimePointer<'a, $type> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                match self
                    .object_store
                    .get_mut(self.object_ref)
                    .expect("Failed to get object from store.")
                {
                    $variant_mut => $result_mut,
                    o => panic!("Unexpected object: {o:?}"),
                }
            }
        }
    };
    ($type:ty, $target:ty, $variant:pat => $result:expr, $variant_mut:pat => $result_mut:expr) => {
        impl<'a> Deref for RuntimePointer<'a, $type> {
            type Target = $target;

            fn deref(&self) -> &Self::Target {
                match self
                    .object_store
                    .get(self.object_ref)
                    .expect("Failed to get object from store.")
                {
                    $variant => $result,
                    o => panic!("Unexpected object: {o:?}"),
                }
            }
        }

        impl<'a> DerefMut for RuntimePointer<'a, $type> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                match self
                    .object_store
                    .get_mut(self.object_ref)
                    .expect("Failed to get object from store.")
                {
                    $variant_mut => $result_mut,
                    o => panic!("Unexpected object: {o:?}"),
                }
            }
        }
    };
}

impl_runtime_pointer_deref!(&str, str, Object::String(s) => s.chars.as_str(), Object::String(s) => s.chars.as_mut());
impl_runtime_pointer_deref!(ObjString, Object::String(s) => s, mut);
impl_runtime_pointer_deref!(String, Object::String(s) => &s.chars, Object::String(s) => &mut s.chars);
impl_runtime_pointer_deref!(ObjBoundMethod, Object::BoundMethod(bm) => bm, mut);
impl_runtime_pointer_deref!(ObjClass, Object::Class(c) => c, mut);
impl_runtime_pointer_deref!(ObjClosure, Object::Closure(c) => c, mut);
impl_runtime_pointer_deref!(ObjInstance, Object::Instance(i) => i, mut);
impl_runtime_pointer_deref!(ObjNative, Object::Native(n) => n);
impl_runtime_pointer_deref!(ObjFunction, Object::Function(f) => f);
impl_runtime_pointer_deref!(ObjUpvalue, Object::UpValue(u) => u, mut);
