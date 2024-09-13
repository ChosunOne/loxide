use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::{
    object::{
        ObjBoundMethod, ObjClass, ObjClosure, ObjFunction, ObjInstance, ObjNative, ObjString,
        ObjUpvalue, Object,
    },
    value::{RuntimeValue, ValueStore},
};

pub struct RuntimePointer<'a, T> {
    pub(super) value_store: &'a mut ValueStore,
    pub(super) value_ref: *const RuntimeValue,
    pub(super) _phantom: PhantomData<T>,
}

impl<'a, T> RuntimePointer<'a, T> {
    pub fn as_raw_ptr(self) -> *const RuntimeValue {
        self.value_ref
    }
}

macro_rules! match_runtime_value {
    ($value:expr, $variant:pat => $result:expr) => {
        match $value {
            $variant => $result,
            v => panic!("Unexpected value: {v}"),
        }
    };
}

macro_rules! impl_runtime_pointer_deref {
    ($type:ty, $variant:pat => $result:expr) => {
        impl<'a> Deref for RuntimePointer<'a, $type> {
            type Target = $type;

            fn deref(&self) -> &Self::Target {
                match_runtime_value!(self.value_store.get(self.value_ref).expect("Failed to get value from store"), $variant => $result)
            }
        }
    };
    ($type:ty, $variant:pat => $result:expr, mut) => {
        impl<'a> Deref for RuntimePointer<'a, $type> {
            type Target = $type;

            fn deref(&self) -> &Self::Target {
                match_runtime_value!(self.value_store.get(self.value_ref).expect("Failed to get value from store"), $variant => $result)
            }
        }

        impl<'a> DerefMut for RuntimePointer<'a, $type> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                match_runtime_value!(self.value_store.get_mut(self.value_ref).expect("Failed to get value from store"), $variant => $result)
            }
        }
    };
}

macro_rules! match_runtime_value_object {
    ($value:expr, $variant:pat => $result:expr) => {
        match $value {
            RuntimeValue::Object(o) => match &**o {
                $variant => $result,
                o => panic!("Unexpected object: {o}"),
            },
            v => panic!("Unexpected value: {v}"),
        }
    };
    ($value:expr, $variant:pat => $result:expr, mut) => {
        match $value {
            RuntimeValue::Object(o) => match &mut **o {
                $variant => $result,
                o => panic!("Unexpected object: {o}"),
            },
            v => panic!("Unexpected value: {v}"),
        }
    };
}

macro_rules! impl_runtime_pointer_object_deref {
    ($type:ty, $target:ty, $variant:pat => $result:expr) => {
        impl<'a> Deref for RuntimePointer<'a, $type> {
            type Target = $target;

            fn deref(&self) -> &Self::Target {
                match_runtime_value_object!(
                    self.value_store.get(self.value_ref).expect("Failed to get value from store."),
                    $variant => $result
                )
            }
        }
    };
    ($type:ty, $target:ty, $variant:pat => $result:expr, mut) => {
        impl<'a> Deref for RuntimePointer<'a, $type> {
            type Target = $target;

            fn deref(&self) -> &Self::Target {
                match_runtime_value_object!(
                    self.value_store.get(self.value_ref).expect("Failed to get value from store."),
                    $variant => $result
                )
            }
        }

        impl<'a> DerefMut for RuntimePointer<'a, $type> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                match_runtime_value_object!(
                    self.value_store.get_mut(self.value_ref).expect("Failed to get value from store."),
                    $variant => $result,
                    mut
                )
            }
        }
    };
    ($type:ty, $variant:pat => $result:expr) => {
        impl<'a> Deref for RuntimePointer<'a, $type> {
            type Target = $type;

            fn deref(&self) -> &Self::Target {
                match_runtime_value_object!(
                    self.value_store.get(self.value_ref).expect("Failed to get value from store."),
                    $variant => $result
                )
            }
        }
    };
    ($type:ty, $variant:pat => $result:expr, mut) => {
        impl<'a> Deref for RuntimePointer<'a, $type> {
            type Target = $type;

            fn deref(&self) -> &Self::Target {
                match_runtime_value_object!(
                    self.value_store.get(self.value_ref).expect("Failed to get value from store."),
                    $variant => $result
                )
            }
        }

        impl<'a> DerefMut for RuntimePointer<'a, $type> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                match_runtime_value_object!(
                    self.value_store.get_mut(self.value_ref).expect("Failed to get value from store."),
                    $variant => $result,
                    mut
                )
            }
        }
    };
    ($type:ty, $target:ty, $variant:pat => $result:expr, $variant_mut:pat => $result_mut:expr) => {
        impl<'a> Deref for RuntimePointer<'a, $type> {
            type Target = $target;

            fn deref(&self) -> &Self::Target {
                match_runtime_value_object!(
                    self.value_store.get(self.value_ref).expect("Failed to get value from store."),
                    $variant => $result
                )
            }
        }

        impl<'a> DerefMut for RuntimePointer<'a, $type> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                match_runtime_value_object!(
                    self.value_store.get_mut(self.value_ref).expect("Failed to get value from store."),
                    $variant_mut => $result_mut,
                    mut
                )
            }
        }
    };
}

impl_runtime_pointer_deref!(f64, RuntimeValue::Number(n) => n, mut);
impl_runtime_pointer_deref!(bool, RuntimeValue::Bool(b) => b, mut);
impl_runtime_pointer_object_deref!(&str, str, Object::String(s) => s.chars.as_str());
impl_runtime_pointer_object_deref!(ObjString, Object::String(s) => s, mut);
impl_runtime_pointer_object_deref!(String, String, Object::String(s) => &s.chars, Object::String(s) => &mut s.chars);
impl_runtime_pointer_object_deref!(ObjBoundMethod, Object::BoundMethod(bm) => bm, mut);
impl_runtime_pointer_object_deref!(ObjClass, Object::Class(c) => c, mut);
impl_runtime_pointer_object_deref!(ObjClosure, Object::Closure(c) => c, mut);
impl_runtime_pointer_object_deref!(ObjInstance, Object::Instance(i) => i, mut);
impl_runtime_pointer_object_deref!(ObjNative, Object::Native(n) => n);
impl_runtime_pointer_object_deref!(ObjFunction, Object::Function(f) => f);
impl_runtime_pointer_object_deref!(ObjUpvalue, Object::UpValue(u) => u, mut);
