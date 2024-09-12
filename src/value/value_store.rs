use std::{collections::BTreeMap, pin::Pin};

use crate::value::RuntimeValue;

#[derive(Default)]
pub struct ValueStore {
    // This is kind of brilliant ngl
    map: BTreeMap<*const RuntimeValue, Pin<Box<RuntimeValue>>>,
}

impl ValueStore {
    pub fn insert(&mut self, value: impl Into<RuntimeValue>) -> *const RuntimeValue {
        let pinned_object = Pin::new(Box::new(value.into()));
        let pinned_reference = &*pinned_object as *const RuntimeValue;
        self.map.insert(pinned_reference, pinned_object);
        pinned_reference
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
    use crate::object::Object;

    use super::*;

    #[test]
    fn it_inserts_and_retrieves_values() {
        let mut value_store = ValueStore::default();
        let value = "test string value";
        let value_ref = value_store.insert(value);
        let retrieved_value = value_store.get(value_ref).unwrap();
        match retrieved_value {
            RuntimeValue::Object(o) => match &**o {
                Object::String(s) => assert_eq!(s.chars, "test string value"),
                o => panic!("Unexpected object: {o}"),
            },
            v => panic!("Unexpected value: {v}"),
        }
    }

    #[test]
    fn it_gets_a_mutable_value() {
        let mut value_store = ValueStore::default();
        let value = "test string value";
        let value_ref = value_store.insert(value);
        {
            let mutable_value = match value_store.get_mut(value_ref).unwrap() {
                RuntimeValue::Object(o) => match &mut **o {
                    Object::String(s) => &mut s.chars,
                    o => panic!("Unexpected object: {o}"),
                },
                v => panic!("Unexpected value: {v}"),
            };

            *mutable_value += " mutated";
        }
        let retrieved_value = value_store.get(value_ref).unwrap();
        match retrieved_value {
            RuntimeValue::Object(o) => match &**o {
                Object::String(s) => assert_eq!(s.chars, "test string value mutated"),
                o => panic!("Unexpected object: {o}"),
            },
            v => panic!("Unexpected value: {v}"),
        }
    }

    #[test]
    fn it_frees_a_value() {
        let mut value_store = ValueStore::default();
        let value = "test string value";
        let value_ref = value_store.insert(value);
        value_store.free(value_ref);
        let retrieved_value = value_store.get(value_ref);
        assert!(retrieved_value.is_none())
    }
}
