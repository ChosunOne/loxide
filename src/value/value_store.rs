use std::{
    collections::HashMap,
    hash::{BuildHasherDefault, Hasher},
    marker::PhantomData,
    pin::Pin,
};

use crate::value::{RuntimePointer, RuntimeValue};

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

#[derive(Default)]
pub struct ValueStore {
    // This is kind of brilliant ngl
    map: HashMap<*const RuntimeValue, Pin<Box<RuntimeValue>>, BuildHasherDefault<PointerHasher>>,
}

impl ValueStore {
    pub fn insert<T: Into<RuntimeValue>>(&mut self, value: T) -> RuntimePointer<T> {
        let pinned_object = Pin::new(Box::new(value.into()));
        let pinned_reference = &*pinned_object as *const RuntimeValue;
        self.map.insert(pinned_reference, pinned_object);
        RuntimePointer::<T> {
            value_store: self,
            value_ref: pinned_reference,
            _phantom: PhantomData,
        }
    }

    pub fn get(&self, key: *const RuntimeValue) -> Option<&RuntimeValue> {
        self.map.get(&key).map(|v| &**v)
    }

    pub fn get_mut(&mut self, key: *const RuntimeValue) -> Option<&mut RuntimeValue> {
        self.map.get_mut(&key).map(|v| &mut **v)
    }

    pub fn free(&mut self, key: *const RuntimeValue) {
        self.map.remove(&key);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_inserts_and_retrieves_strings() {
        let mut value_store = ValueStore::default();
        let value = "test string value".to_owned();
        let value_ref = value_store.insert(value);
        let retrieved_value = &*value_ref;
        assert_eq!(retrieved_value, "test string value")
    }

    #[test]
    fn it_gets_a_mutable_string() {
        let mut value_store = ValueStore::default();
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
        let mut value_store = ValueStore::default();
        let value = "test string value";
        let value_ref = value_store.insert(value).as_raw_ptr();
        value_store.free(value_ref);
        let retrieved_value = value_store.get(value_ref);
        assert!(retrieved_value.is_none())
    }
}
