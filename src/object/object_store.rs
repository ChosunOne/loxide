use std::{
    collections::HashMap,
    hash::{BuildHasherDefault, Hash, Hasher},
    ops::{Deref, DerefMut},
    pin::Pin,
    ptr::NonNull,
};

use crate::{error::Error, value::RuntimeValue};

use super::{ObjBoundMethod, ObjClass, ObjClosure, ObjFunction, ObjInstance, ObjNative, ObjString};

#[derive(Default)]
struct PointerHasher(u64);

impl Hasher for PointerHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0 = u64::from_ne_bytes(bytes.try_into().unwrap())
    }
}

#[derive(Debug)]
pub struct Pointer<T>(NonNull<T>);

impl TryFrom<RuntimeValue> for Pointer<ObjBoundMethod> {
    type Error = Error;

    fn try_from(value: RuntimeValue) -> Result<Self, Self::Error> {
        match value {
            RuntimeValue::BoundMethod(pointer) => Ok(pointer),
            _ => Err(Error::Runtime),
        }
    }
}

impl TryFrom<RuntimeValue> for Pointer<ObjClass> {
    type Error = Error;

    fn try_from(value: RuntimeValue) -> Result<Self, Self::Error> {
        match value {
            RuntimeValue::Class(pointer) => Ok(pointer),
            _ => Err(Error::Runtime),
        }
    }
}

impl TryFrom<RuntimeValue> for Pointer<ObjClosure> {
    type Error = Error;

    fn try_from(value: RuntimeValue) -> Result<Self, Self::Error> {
        match value {
            RuntimeValue::Closure(pointer) => Ok(pointer),
            _ => Err(Error::Runtime),
        }
    }
}

impl TryFrom<RuntimeValue> for Pointer<ObjFunction> {
    type Error = Error;

    fn try_from(value: RuntimeValue) -> Result<Self, Self::Error> {
        match value {
            RuntimeValue::Function(pointer) => Ok(pointer),
            _ => Err(Error::Runtime),
        }
    }
}

impl TryFrom<RuntimeValue> for Pointer<ObjInstance> {
    type Error = Error;

    fn try_from(value: RuntimeValue) -> Result<Self, Self::Error> {
        match value {
            RuntimeValue::Instance(pointer) => Ok(pointer),
            _ => Err(Error::Runtime),
        }
    }
}

impl TryFrom<RuntimeValue> for Pointer<ObjNative> {
    type Error = Error;

    fn try_from(value: RuntimeValue) -> Result<Self, Self::Error> {
        match value {
            RuntimeValue::Native(pointer) => Ok(pointer),
            _ => Err(Error::Runtime),
        }
    }
}

impl TryFrom<RuntimeValue> for Pointer<ObjString> {
    type Error = Error;

    fn try_from(value: RuntimeValue) -> Result<Self, Self::Error> {
        match value {
            RuntimeValue::String(pointer) => Ok(pointer),
            _ => Err(Error::Runtime),
        }
    }
}

impl<T> Clone for Pointer<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Pointer<T> {}

impl<T> Deref for Pointer<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl<T> DerefMut for Pointer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut() }
    }
}

impl<T> PartialEq for Pointer<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T> Eq for Pointer<T> {}

impl<T> Hash for Pointer<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

#[derive(Debug)]
pub struct ObjectStore<T> {
    map: HashMap<Pointer<T>, Pin<Box<T>>, BuildHasherDefault<PointerHasher>>,
}

impl<T> ObjectStore<T> {
    pub fn insert(&mut self, value: T) -> Pointer<T> {
        let pinned_object = Box::pin(value);
        let pinned_ref = Pointer(NonNull::from(&*pinned_object));
        self.map.insert(pinned_ref, pinned_object);
        pinned_ref
    }

    pub fn insert_pinned(&mut self, value: Pin<Box<T>>) -> Pointer<T> {
        let pointer = Pointer(NonNull::from(&*value));
        self.map.insert(pointer, value);
        pointer
    }

    pub fn free(&mut self, key: Pointer<T>) {
        self.map.remove(&key);
    }
}

impl<T> Default for ObjectStore<T> {
    fn default() -> Self {
        Self {
            map: HashMap::<Pointer<T>, Pin<Box<T>>, BuildHasherDefault<PointerHasher>>::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_inserts_and_retrieves_strings() {
        let mut value_store = ObjectStore::default();
        let value = "test string value";
        let value_ref = value_store.insert(value);
        let retrieved_value = *value_ref;
        assert_eq!(retrieved_value, "test string value")
    }

    #[test]
    fn it_gets_a_mutable_string() {
        let mut value_store = ObjectStore::default();
        let value = "test string value".to_owned();
        let mut value_ref = value_store.insert(value);
        {
            *value_ref += " mutated";
        }
        let retrieved_value = &*value_ref;
        assert_eq!(retrieved_value, "test string value mutated")
    }

    #[test]
    fn it_frees_a_value() {
        let mut value_store = ObjectStore::default();
        let value = "test string value";
        let value_ref = value_store.insert(value);
        value_store.free(value_ref);
        let retrieved_value = value_store.map.get(&value_ref);
        assert!(retrieved_value.is_none())
    }
}
