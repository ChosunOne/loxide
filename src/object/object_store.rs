use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    hash::{BuildHasherDefault, Hash, Hasher},
    ops::{Deref, DerefMut},
    pin::Pin,
    ptr::NonNull,
};

use crate::{error::Error, value::RuntimeValue};

use super::{
    HeapSize, ObjBoundMethod, ObjClass, ObjClosure, ObjFunction, ObjInstance, ObjNative, ObjString,
    ObjUpvalue,
};

#[derive(Default)]
pub struct PointerHasher(u64);

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

impl<T> Default for Pointer<T> {
    fn default() -> Self {
        Self(NonNull::dangling())
    }
}

impl<T> Clone for Pointer<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Pointer<T> {}

impl Display for Pointer<ObjBoundMethod> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", unsafe { self.0.as_ref() })
    }
}

impl Display for Pointer<ObjClass> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", unsafe { self.0.as_ref() })
    }
}

impl Display for Pointer<ObjClosure> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", unsafe { self.0.as_ref() })
    }
}

impl Display for Pointer<ObjFunction> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", unsafe { self.0.as_ref() })
    }
}

impl Display for Pointer<ObjInstance> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", unsafe { self.0.as_ref() })
    }
}

impl Display for Pointer<ObjNative> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", unsafe { self.0.as_ref() })
    }
}

impl Display for Pointer<ObjString> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", unsafe { self.0.as_ref() })
    }
}

impl Display for Pointer<ObjUpvalue> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", unsafe { self.0.as_ref() })
    }
}

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
        self.0.as_ptr().eq(&other.0.as_ptr())
    }
}

impl<T> Eq for Pointer<T> {}

impl<T> Hash for Pointer<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.as_ptr().hash(state)
    }
}

impl<T> HeapSize for Pointer<T> {
    fn size(&self) -> usize {
        size_of::<Pointer<T>>()
    }
}

#[derive(Debug)]
pub struct ObjectStore<T> {
    map: HashMap<NonNull<T>, Pin<Box<T>>, BuildHasherDefault<PointerHasher>>,
}

impl<T: Debug + HeapSize> ObjectStore<T> {
    pub fn insert(&mut self, value: T) -> Pointer<T> {
        let value_box = Box::pin(value);
        let value_ptr = NonNull::from(&*value_box);
        self.map.insert(value_ptr, value_box);
        Pointer(value_ptr)
    }

    pub fn free(&mut self, key: Pointer<T>) -> usize {
        let Some(o) = self.map.remove(&key.0) else {
            return 0;
        };
        o.size()
    }

    pub fn keys(&self) -> Vec<Pointer<T>> {
        self.map.keys().map(|x| Pointer(*x)).collect()
    }

    pub fn contains_key(&self, key: &Pointer<T>) -> bool {
        self.map.contains_key(&key.0)
    }
}

impl<T> Default for ObjectStore<T> {
    fn default() -> Self {
        Self {
            map: HashMap::<NonNull<T>, Pin<Box<T>>, BuildHasherDefault<PointerHasher>>::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_inserts_and_retrieves_strings() {
        let mut value_store = ObjectStore::<ObjString>::default();
        let value = "test string value".into();
        let value_ref = value_store.insert(value);
        let retrieved_value = &value_ref.chars;
        assert_eq!(retrieved_value, "test string value");
    }

    #[test]
    fn it_gets_a_mutable_string() {
        let mut value_store = ObjectStore::<ObjString>::default();
        let value = "test string value".into();
        let mut value_ref = value_store.insert(value);
        {
            value_ref.chars += " mutated";
        }
        let retrieved_value = &value_ref.chars;
        assert_eq!(retrieved_value, "test string value mutated");
    }

    #[test]
    fn it_frees_a_value() {
        let mut value_store = ObjectStore::<ObjString>::default();
        let value = "test string value".into();
        let value_ref = value_store.insert(value);
        let freed_bytes = value_store.free(value_ref);
        let retrieved_value = value_store.map.get(&value_ref.0);
        assert!(retrieved_value.is_none());
        assert_eq!(
            freed_bytes,
            "test string value".to_owned().len() + size_of::<String>() + size_of::<u32>()
        );
    }
}
