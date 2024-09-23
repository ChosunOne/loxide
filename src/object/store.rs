use std::{cell::RefCell, rc::Rc};

use super::{
    ObjBoundMethod, ObjClass, ObjClosure, ObjFunction, ObjInstance, ObjNative, ObjString,
    ObjUpvalue, ObjectStore, Pointer,
};

#[derive(Debug, Default)]
pub struct Store {
    pub bound_method_store: ObjectStore<ObjBoundMethod>,
    pub class_store: ObjectStore<ObjClass>,
    pub closure_store: ObjectStore<ObjClosure>,
    pub function_store: ObjectStore<ObjFunction>,
    pub instance_store: ObjectStore<ObjInstance>,
    pub native_store: ObjectStore<ObjNative>,
    pub string_store: ObjectStore<ObjString>,
    pub upvalue_store: ObjectStore<ObjUpvalue>,
}

impl Store {
    pub fn insert_bound_method(&mut self, bound_method: ObjBoundMethod) -> Pointer<ObjBoundMethod> {
        self.bound_method_store.insert(bound_method)
    }

    pub fn insert_bound_method_pointer(
        &mut self,
        function: Rc<RefCell<ObjBoundMethod>>,
    ) -> Pointer<ObjBoundMethod> {
        self.bound_method_store.insert_pointer(function)
    }

    pub fn insert_class(&mut self, class: ObjClass) -> Pointer<ObjClass> {
        self.class_store.insert(class)
    }

    pub fn insert_class_pointer(&mut self, class: Rc<RefCell<ObjClass>>) -> Pointer<ObjClass> {
        self.class_store.insert_pointer(class)
    }

    pub fn insert_closure(&mut self, closure: ObjClosure) -> Pointer<ObjClosure> {
        self.closure_store.insert(closure)
    }

    pub fn insert_closure_pointer(
        &mut self,
        closure: Rc<RefCell<ObjClosure>>,
    ) -> Pointer<ObjClosure> {
        self.closure_store.insert_pointer(closure)
    }

    pub fn insert_function(&mut self, function: ObjFunction) -> Pointer<ObjFunction> {
        self.function_store.insert(function)
    }

    pub fn insert_function_pointer(
        &mut self,
        function: Rc<RefCell<ObjFunction>>,
    ) -> Pointer<ObjFunction> {
        self.function_store.insert_pointer(function)
    }

    pub fn insert_instance(&mut self, instance: ObjInstance) -> Pointer<ObjInstance> {
        self.instance_store.insert(instance)
    }

    pub fn insert_instance_pointer(
        &mut self,
        instance: Rc<RefCell<ObjInstance>>,
    ) -> Pointer<ObjInstance> {
        self.instance_store.insert_pointer(instance)
    }

    pub fn insert_native(&mut self, native: ObjNative) -> Pointer<ObjNative> {
        self.native_store.insert(native)
    }

    pub fn insert_native_pointer(&mut self, native: Rc<RefCell<ObjNative>>) -> Pointer<ObjNative> {
        self.native_store.insert_pointer(native)
    }

    pub fn insert_string(&mut self, string: ObjString) -> Pointer<ObjString> {
        self.string_store.insert(string)
    }

    pub fn insert_string_pointer(&mut self, string: Rc<RefCell<ObjString>>) -> Pointer<ObjString> {
        self.string_store.insert_pointer(string)
    }

    pub fn insert_upvalue(&mut self, upvalue: ObjUpvalue) -> Pointer<ObjUpvalue> {
        self.upvalue_store.insert(upvalue)
    }

    pub fn insert_upvalue_pointer(
        &mut self,
        upvalue: Rc<RefCell<ObjUpvalue>>,
    ) -> Pointer<ObjUpvalue> {
        self.upvalue_store.insert_pointer(upvalue)
    }
}
