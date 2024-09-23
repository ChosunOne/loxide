use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::Display,
    hash::{BuildHasherDefault, Hash, Hasher},
    ops::Deref,
    rc::Rc,
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
pub struct Pointer<T>(Rc<RefCell<T>>);

impl Display for Pointer<ObjBoundMethod> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.borrow())
    }
}

impl Display for Pointer<ObjClass> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.borrow())
    }
}

impl Display for Pointer<ObjClosure> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.borrow())
    }
}

impl Display for Pointer<ObjFunction> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.borrow())
    }
}

impl Display for Pointer<ObjInstance> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.borrow())
    }
}

impl Display for Pointer<ObjNative> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.borrow())
    }
}

impl Display for Pointer<ObjString> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.borrow())
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

impl<T> Clone for Pointer<T> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<T> Deref for Pointer<T> {
    type Target = RefCell<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
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

#[derive(Debug)]
pub struct ObjectStore<T> {
    map: HashMap<Pointer<T>, Rc<RefCell<T>>, BuildHasherDefault<PointerHasher>>,
}

impl<T> ObjectStore<T> {
    pub fn insert(&mut self, value: T) -> Pointer<T> {
        let object = Rc::new(RefCell::new(value));
        let pointer = Pointer(Rc::clone(&object));
        self.map.insert(pointer.clone(), object);
        pointer
    }

    pub fn insert_pointer(&mut self, value: Rc<RefCell<T>>) -> Pointer<T> {
        let pointer = Pointer(Rc::clone(&value));
        self.map.insert(pointer.clone(), value);
        pointer
    }

    pub fn free(&mut self, key: Pointer<T>) {
        self.map.remove(&key);
    }
}

impl<T> Default for ObjectStore<T> {
    fn default() -> Self {
        Self {
            map: HashMap::<Pointer<T>, Rc<RefCell<T>>, BuildHasherDefault<PointerHasher>>::default(
            ),
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
        let retrieved_value = *value_ref.borrow();
        assert_eq!(retrieved_value, "test string value");
    }

    #[test]
    fn it_gets_a_mutable_string() {
        let mut value_store = ObjectStore::default();
        let value = "test string value".to_owned();
        let value_ref = value_store.insert(value);
        {
            *value_ref.borrow_mut() += " mutated";
        }
        let retrieved_value = value_ref.borrow().clone();
        assert_eq!(retrieved_value, "test string value mutated")
    }

    #[test]
    fn it_frees_a_value() {
        let mut value_store = ObjectStore::default();
        let value = "test string value";
        let value_ref = value_store.insert(value);
        value_store.free(value_ref.clone());
        let retrieved_value = value_store.map.get(&value_ref);
        assert!(retrieved_value.is_none())
    }
}
