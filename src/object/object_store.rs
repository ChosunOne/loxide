use std::{
    collections::HashMap,
    hash::{BuildHasherDefault, Hasher},
    marker::PhantomData,
    pin::Pin,
};

use crate::{
    object::{
        ObjBoundMethod, ObjClass, ObjClosure, ObjFunction, ObjInstance, ObjNative, ObjString,
        ObjUpvalue, Object,
    },
    value::{RuntimePointer, RuntimeReference},
};

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

pub trait GetPointer<T> {
    fn get_pointer(&mut self, key: RuntimeReference<T>) -> Option<RuntimePointer<T>>;
}

#[derive(Default, Debug)]
pub struct ObjectStore {
    // This is kind of brilliant ngl
    map: HashMap<*const Object, Pin<Box<Object>>, BuildHasherDefault<PointerHasher>>,
}

impl ObjectStore {
    pub fn insert<T: Into<Object>>(&mut self, value: T) -> RuntimePointer<T> {
        let pinned_object = Box::pin(value.into());
        let pinned_reference = &*pinned_object as *const Object;
        self.map.insert(pinned_reference, pinned_object);
        RuntimePointer::<T> {
            object_store: self,
            object_ref: pinned_reference,
            _phantom: PhantomData,
        }
    }

    pub fn insert_pinned(&mut self, value: Pin<Box<Object>>) {
        let pinned_reference = &*value as *const Object;
        self.map.insert(pinned_reference, value);
    }

    pub fn get(&self, key: *const Object) -> Option<&Object> {
        self.map.get(&key).map(|v| &**v)
    }

    pub fn get_mut(&mut self, key: *const Object) -> Option<&mut Object> {
        self.map.get_mut(&key).map(|v| &mut **v)
    }

    pub fn free(&mut self, key: *const Object) {
        self.map.remove(&key);
    }
}

impl GetPointer<String> for ObjectStore {
    fn get_pointer(&mut self, key: RuntimeReference<String>) -> Option<RuntimePointer<String>> {
        match self.get(key.object_ref)? {
            Object::String(_) => Some(RuntimePointer {
                object_store: self,
                object_ref: key.object_ref,
                _phantom: PhantomData,
            }),
            _ => None,
        }
    }
}

impl GetPointer<ObjString> for ObjectStore {
    fn get_pointer(
        &mut self,
        key: RuntimeReference<ObjString>,
    ) -> Option<RuntimePointer<ObjString>> {
        match self.get(key.object_ref)? {
            Object::String(_) => Some(RuntimePointer {
                object_store: self,
                object_ref: key.object_ref,
                _phantom: PhantomData,
            }),
            _ => None,
        }
    }
}

impl GetPointer<ObjBoundMethod> for ObjectStore {
    fn get_pointer(
        &mut self,
        key: RuntimeReference<ObjBoundMethod>,
    ) -> Option<RuntimePointer<ObjBoundMethod>> {
        match self.get(key.object_ref)? {
            Object::BoundMethod(_) => Some(RuntimePointer {
                object_store: self,
                object_ref: key.object_ref,
                _phantom: PhantomData,
            }),
            _ => None,
        }
    }
}

impl GetPointer<ObjClass> for ObjectStore {
    fn get_pointer(&mut self, key: RuntimeReference<ObjClass>) -> Option<RuntimePointer<ObjClass>> {
        match self.get(key.object_ref)? {
            Object::Class(_) => Some(RuntimePointer {
                object_store: self,
                object_ref: key.object_ref,
                _phantom: PhantomData,
            }),
            _ => None,
        }
    }
}

impl GetPointer<ObjClosure> for ObjectStore {
    fn get_pointer(
        &mut self,
        key: RuntimeReference<ObjClosure>,
    ) -> Option<RuntimePointer<ObjClosure>> {
        match self.get(key.object_ref)? {
            Object::Closure(_) => Some(RuntimePointer {
                object_store: self,
                object_ref: key.object_ref,
                _phantom: PhantomData,
            }),
            _ => None,
        }
    }
}

impl GetPointer<ObjFunction> for ObjectStore {
    fn get_pointer(
        &mut self,
        key: RuntimeReference<ObjFunction>,
    ) -> Option<RuntimePointer<ObjFunction>> {
        match self.get(key.object_ref)? {
            Object::Function(_) => Some(RuntimePointer {
                object_store: self,
                object_ref: key.object_ref,
                _phantom: PhantomData,
            }),
            _ => None,
        }
    }
}

impl GetPointer<ObjInstance> for ObjectStore {
    fn get_pointer(
        &mut self,
        key: RuntimeReference<ObjInstance>,
    ) -> Option<RuntimePointer<ObjInstance>> {
        match self.get(key.object_ref)? {
            Object::Instance(_) => Some(RuntimePointer {
                object_store: self,
                object_ref: key.object_ref,
                _phantom: PhantomData,
            }),
            _ => None,
        }
    }
}

impl GetPointer<ObjNative> for ObjectStore {
    fn get_pointer(
        &mut self,
        key: RuntimeReference<ObjNative>,
    ) -> Option<RuntimePointer<ObjNative>> {
        match self.get(key.object_ref)? {
            Object::Native(_) => Some(RuntimePointer {
                object_store: self,
                object_ref: key.object_ref,
                _phantom: PhantomData,
            }),
            _ => None,
        }
    }
}

impl GetPointer<ObjUpvalue> for ObjectStore {
    fn get_pointer(
        &mut self,
        key: RuntimeReference<ObjUpvalue>,
    ) -> Option<RuntimePointer<ObjUpvalue>> {
        match self.get(key.object_ref)? {
            Object::UpValue(_) => Some(RuntimePointer {
                object_store: self,
                object_ref: key.object_ref,
                _phantom: PhantomData,
            }),
            _ => None,
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
        let retrieved_value = &*value_ref;
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
        let value_ref = value_store.insert(value).as_raw_ptr();
        value_store.free(value_ref);
        let retrieved_value = value_store.get(value_ref);
        assert!(retrieved_value.is_none())
    }

    #[test]
    fn it_makes_a_string_reference_from_a_pointer() {
        let mut value_store = ObjectStore::default();
        let value = "test string value".to_owned();
        let value_ptr = value_store.insert(value);
        let value_ref: RuntimeReference<String> = (&value_ptr).into();
        let value_ptr = value_store.get_pointer(value_ref).unwrap();
        assert_eq!(&*value_ptr, "test string value");
    }
}
