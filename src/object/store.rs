use std::{
    array,
    cell::RefCell,
    collections::{BTreeMap, HashSet},
    fmt::Debug,
    rc::Rc,
};

use crate::{call_frame::CallFrame, table::Table, value::RuntimeValue, vm::MAX_FRAMES};

use super::{
    HeapSize, ObjBoundMethod, ObjClass, ObjClosure, ObjFunction, ObjInstance, ObjNative, ObjString,
    ObjUpvalue, ObjectStore, Pointer,
};

const GC_HEAP_GROW_FACTOR: usize = 2;
pub const MAX_STACK_SIZE: usize = 128 * MAX_FRAMES;

#[derive(Debug)]
pub struct Store {
    pub bound_method_store: ObjectStore<ObjBoundMethod>,
    pub class_store: ObjectStore<ObjClass>,
    pub closure_store: ObjectStore<ObjClosure>,
    pub function_store: ObjectStore<ObjFunction>,
    pub instance_store: ObjectStore<ObjInstance>,
    pub native_store: ObjectStore<ObjNative>,
    pub string_store: ObjectStore<ObjString>,
    pub upvalue_store: ObjectStore<ObjUpvalue>,
    pub value_stack: Vec<RuntimeValue>,
    pub frame_stack: [CallFrame; MAX_FRAMES],
    pub frame_stack_top: usize,
    pub open_upvalues: BTreeMap<usize, Pointer<ObjUpvalue>>,
    pub globals: Table<RuntimeValue>,
    bytes_allocated: usize,
    next_gc: usize,
}

impl Default for Store {
    fn default() -> Self {
        Self {
            bound_method_store: ObjectStore::<ObjBoundMethod>::default(),
            class_store: ObjectStore::<ObjClass>::default(),
            closure_store: ObjectStore::<ObjClosure>::default(),
            function_store: ObjectStore::<ObjFunction>::default(),
            instance_store: ObjectStore::<ObjInstance>::default(),
            native_store: ObjectStore::<ObjNative>::default(),
            string_store: ObjectStore::<ObjString>::default(),
            upvalue_store: ObjectStore::<ObjUpvalue>::default(),
            globals: Table::default(),
            value_stack: Vec::with_capacity(MAX_STACK_SIZE),
            frame_stack: array::from_fn(|_| CallFrame::default()),
            frame_stack_top: 0,
            open_upvalues: BTreeMap::default(),
            next_gc: 1024 * 1024,
            bytes_allocated: 0,
        }
    }
}

impl Store {
    pub fn insert_bound_method(&mut self, bound_method: ObjBoundMethod) -> Pointer<ObjBoundMethod> {
        self.bytes_allocated += bound_method.size();
        self.collect_garbage();
        self.bound_method_store.insert(bound_method)
    }

    pub fn insert_bound_method_pointer(
        &mut self,
        function: Rc<RefCell<ObjBoundMethod>>,
    ) -> Pointer<ObjBoundMethod> {
        self.bytes_allocated += function.borrow().size();
        self.collect_garbage();
        self.bound_method_store.insert_pointer(function)
    }

    pub fn insert_class(&mut self, class: ObjClass) -> Pointer<ObjClass> {
        self.bytes_allocated += class.size();
        self.collect_garbage();
        self.class_store.insert(class)
    }

    pub fn insert_class_pointer(&mut self, class: Rc<RefCell<ObjClass>>) -> Pointer<ObjClass> {
        self.bytes_allocated += class.borrow().size();
        self.collect_garbage();
        self.class_store.insert_pointer(class)
    }

    pub fn insert_closure(&mut self, closure: ObjClosure) -> Pointer<ObjClosure> {
        self.bytes_allocated += closure.size();
        self.collect_garbage();
        self.closure_store.insert(closure)
    }

    pub fn insert_closure_pointer(
        &mut self,
        closure: Rc<RefCell<ObjClosure>>,
    ) -> Pointer<ObjClosure> {
        self.bytes_allocated += closure.borrow().size();
        self.collect_garbage();
        self.closure_store.insert_pointer(closure)
    }

    pub fn insert_function(&mut self, function: ObjFunction) -> Pointer<ObjFunction> {
        self.bytes_allocated += function.size();
        self.collect_garbage();
        self.function_store.insert(function)
    }

    pub fn insert_function_pointer(
        &mut self,
        function: Rc<RefCell<ObjFunction>>,
    ) -> Pointer<ObjFunction> {
        self.bytes_allocated += function.borrow().size();
        self.collect_garbage();
        self.function_store.insert_pointer(function)
    }

    pub fn insert_instance(&mut self, instance: ObjInstance) -> Pointer<ObjInstance> {
        self.bytes_allocated += instance.size();
        self.collect_garbage();
        self.instance_store.insert(instance)
    }

    pub fn insert_instance_pointer(
        &mut self,
        instance: Rc<RefCell<ObjInstance>>,
    ) -> Pointer<ObjInstance> {
        self.bytes_allocated += instance.borrow().size();
        self.collect_garbage();
        self.instance_store.insert_pointer(instance)
    }

    pub fn insert_native(&mut self, native: ObjNative) -> Pointer<ObjNative> {
        self.bytes_allocated += native.size();
        self.collect_garbage();
        self.native_store.insert(native)
    }

    pub fn insert_native_pointer(&mut self, native: Rc<RefCell<ObjNative>>) -> Pointer<ObjNative> {
        self.bytes_allocated += native.borrow().size();
        self.collect_garbage();
        self.native_store.insert_pointer(native)
    }

    pub fn insert_string(&mut self, string: ObjString) -> Pointer<ObjString> {
        self.bytes_allocated += string.size();
        self.collect_garbage();
        self.string_store.insert(string)
    }

    pub fn insert_string_pointer(&mut self, string: Rc<RefCell<ObjString>>) -> Pointer<ObjString> {
        self.bytes_allocated += string.borrow().size();
        self.collect_garbage();
        self.string_store.insert_pointer(string)
    }

    pub fn insert_upvalue(&mut self, upvalue: ObjUpvalue) -> Pointer<ObjUpvalue> {
        self.bytes_allocated += upvalue.size();
        self.collect_garbage();
        self.upvalue_store.insert(upvalue)
    }

    pub fn insert_upvalue_pointer(
        &mut self,
        upvalue: Rc<RefCell<ObjUpvalue>>,
    ) -> Pointer<ObjUpvalue> {
        self.bytes_allocated += upvalue.borrow().size();
        let pointer = self.upvalue_store.insert_pointer(upvalue);
        self.collect_garbage();
        pointer
    }

    fn collect_garbage(&mut self) {
        if self.bytes_allocated <= self.next_gc {
            return;
        }
        #[cfg(feature = "debug")]
        {
            println!("-- gc begin");
        }
        #[cfg(feature = "debug")]
        let before = self.bytes_allocated;

        #[allow(clippy::mutable_key_type)]
        let mut reachable_objects = HashSet::<RuntimeValue>::new();
        let mut tracing_stack = Vec::<RuntimeValue>::new();
        self.mark_roots(&mut reachable_objects, &mut tracing_stack);
        self.trace_references(&mut reachable_objects, tracing_stack);
        self.sweep(reachable_objects);
        self.next_gc = self.bytes_allocated * GC_HEAP_GROW_FACTOR;

        #[cfg(feature = "debug")]
        {
            println!("-- gc end");
            println!(
                "collected {} bytes (from {} to {}) next at {}",
                before - self.bytes_allocated,
                before,
                self.bytes_allocated,
                self.next_gc
            );
        }
    }

    #[allow(clippy::mutable_key_type)]
    fn mark_roots(
        &self,
        reachable_objects: &mut HashSet<RuntimeValue>,
        tracing_stack: &mut Vec<RuntimeValue>,
    ) {
        for value in &self.value_stack {
            mark_value(value.clone(), reachable_objects, tracing_stack);
        }
        for frame in &self.frame_stack[..self.frame_stack_top] {
            let closure = frame
                .closure
                .clone()
                .expect("IVME: Failed to get frame closure");
            mark_value(closure.clone(), reachable_objects, tracing_stack);
        }
        for (_, upvalue) in self.open_upvalues.iter() {
            mark_value(upvalue.clone(), reachable_objects, tracing_stack);
        }

        for value in self.globals.values() {
            mark_value(value.clone(), reachable_objects, tracing_stack);
        }
    }

    #[allow(clippy::mutable_key_type)]
    fn trace_references(
        &self,
        reachable_objects: &mut HashSet<RuntimeValue>,
        mut tracing_stack: Vec<RuntimeValue>,
    ) {
        while let Some(value) = tracing_stack.pop() {
            match value {
                RuntimeValue::BoundMethod(pointer) => {
                    let receiver = pointer.borrow().receiver.clone();
                    mark_value(receiver, reachable_objects, &mut tracing_stack);
                    let method = pointer.borrow().method.clone();
                    mark_value(method, reachable_objects, &mut tracing_stack);
                }
                RuntimeValue::Class(pointer) => {
                    let name = pointer.borrow().name.clone();
                    mark_value(name, reachable_objects, &mut tracing_stack);
                    for method in pointer.borrow().methods.values() {
                        mark_value(method.clone(), reachable_objects, &mut tracing_stack);
                    }
                }
                RuntimeValue::Closure(pointer) => {
                    let function = pointer.borrow().function.clone();
                    mark_value(function, reachable_objects, &mut tracing_stack);

                    for upvalue in pointer.borrow().upvalues.iter() {
                        mark_value(upvalue.clone(), reachable_objects, &mut tracing_stack);
                    }
                }
                RuntimeValue::Instance(pointer) => {
                    let class = pointer.borrow().class.clone();
                    mark_value(class, reachable_objects, &mut tracing_stack);
                    for field in pointer.borrow().fields.values() {
                        mark_value(field.clone(), reachable_objects, &mut tracing_stack);
                    }
                }
                RuntimeValue::Upvalue(pointer) => {
                    if let ObjUpvalue::Closed { value } = &*pointer.borrow() {
                        mark_value(value.clone(), reachable_objects, &mut tracing_stack);
                    }
                }
                _ => continue,
            }
        }
    }

    #[allow(clippy::mutable_key_type)]
    fn sweep(&mut self, reachable_objects: HashSet<RuntimeValue>) {
        self.bytes_allocated -= sweep_store(&mut self.bound_method_store, &reachable_objects)
            + sweep_store(&mut self.class_store, &reachable_objects)
            + sweep_store(&mut self.closure_store, &reachable_objects)
            + sweep_store(&mut self.function_store, &reachable_objects)
            + sweep_store(&mut self.instance_store, &reachable_objects)
            + sweep_store(&mut self.native_store, &reachable_objects)
            + sweep_store(&mut self.string_store, &reachable_objects)
            + sweep_store(&mut self.upvalue_store, &reachable_objects);
    }
}

#[allow(clippy::mutable_key_type)]
fn mark_value(
    value: impl Into<RuntimeValue>,
    reachable_objects: &mut HashSet<RuntimeValue>,
    tracing_stack: &mut Vec<RuntimeValue>,
) {
    let rv = value.into();
    if reachable_objects.contains(&rv) {
        return;
    }
    reachable_objects.insert(rv.clone());
    tracing_stack.push(rv);
}

#[allow(clippy::mutable_key_type)]
fn sweep_store<T: Debug + HeapSize>(
    store: &mut ObjectStore<T>,
    reachable_objects: &HashSet<RuntimeValue>,
) -> usize
where
    RuntimeValue: From<Pointer<T>>,
{
    let mut bytes_freed = 0;
    let mut objects_to_free = Vec::new();
    let keys = store.keys();
    for key in keys {
        if !reachable_objects.contains(&key.clone().into()) {
            objects_to_free.push(key);
        }
    }

    for key in objects_to_free {
        bytes_freed += store.free(key);
    }
    bytes_freed
}

#[cfg(test)]
mod test {
    use crate::table::Table;

    use super::*;

    #[test]
    fn it_runs_the_garbage_collector_strings() {
        let mut store = Store::default();
        let string: ObjString = "test string".into();
        let string_size = string.size();
        let mut allocated_size = 0;
        let mut next_gc = 128;
        store.next_gc = next_gc;
        for _ in 0..100 {
            let pointer = store.insert_string(string.clone());
            allocated_size += string_size;
            if allocated_size > next_gc {
                allocated_size = string_size;
                next_gc = allocated_size * GC_HEAP_GROW_FACTOR;
            }
            assert_eq!(store.bytes_allocated, allocated_size);
            assert_eq!(store.next_gc, next_gc);
            assert!(store.string_store.contains_key(&pointer));
        }
    }

    #[test]
    fn it_preserves_values_on_the_stack() {
        let mut store = Store::default();
        let string = "should be preserved".into();
        let string_to_remove = "should be removed".into();
        let pointer = store.insert_string(string);
        let pointer_to_remove = store.insert_string(string_to_remove);
        store
            .value_stack
            .push(RuntimeValue::String(pointer.clone()));
        store.next_gc = 0;
        store.collect_garbage();
        assert!(store.string_store.contains_key(&pointer));
        assert!(!store.string_store.contains_key(&pointer_to_remove));
    }

    #[test]
    fn it_preserves_globals() {
        let mut store = Store::default();
        let string = "should be preserved".into();
        let pointer = store.insert_string(string);
        store.globals.insert("a".into(), pointer.clone().into());
        store.next_gc = 0;
        store.collect_garbage();
        assert!(store.string_store.contains_key(&pointer));
    }

    #[test]
    fn it_preserves_upvalues() {
        let mut store = Store::default();
        let string = "should be preserved".into();
        let pointer = store.insert_string(string);
        let upvalue = ObjUpvalue::Closed {
            value: pointer.clone().into(),
        };
        let upvalue_pointer = store.insert_upvalue(upvalue);
        store.open_upvalues.insert(1, upvalue_pointer.clone());
        store.next_gc = 0;
        store.collect_garbage();
        assert!(store.string_store.contains_key(&pointer));
        assert!(store.upvalue_store.contains_key(&upvalue_pointer));
    }

    #[test]
    fn it_preserves_call_frame_values() {
        let mut store = Store::default();
        let function = ObjFunction::default();
        let function_pointer = store.insert_function(function);
        let closure = ObjClosure {
            function: function_pointer.clone(),
            upvalues: Vec::new(),
        };
        let closure_pointer = store.insert_closure(closure);
        store.frame_stack[0] = CallFrame {
            closure: Some(closure_pointer.clone()),
            chunk: function_pointer.borrow().chunk.clone(),
            ip: 0,
            slots: 0,
            start_stack_index: 0,
        };
        store.frame_stack_top += 1;
        store.next_gc = 0;
        store.collect_garbage();
        assert!(store.function_store.contains_key(&function_pointer));
        assert!(store.closure_store.contains_key(&closure_pointer));
    }

    #[test]
    fn it_traces_bound_methods() {
        let mut store = Store::default();
        let function = ObjFunction::default();
        let function_pointer = store.insert_function(function);
        let closure = ObjClosure {
            function: function_pointer.clone(),
            upvalues: Vec::new(),
        };
        let closure_pointer = store.insert_closure(closure);
        let bound_method = ObjBoundMethod {
            receiver: RuntimeValue::Nil,
            method: closure_pointer.clone(),
        };
        let bound_method_pointer = store.insert_bound_method(bound_method);
        store
            .globals
            .insert("test".into(), bound_method_pointer.clone().into());
        store.next_gc = 0;
        store.collect_garbage();
        assert!(store.function_store.contains_key(&function_pointer));
        assert!(store.closure_store.contains_key(&closure_pointer));
        assert!(store.bound_method_store.contains_key(&bound_method_pointer));
    }

    #[test]
    fn it_traces_classes() {
        let mut store = Store::default();
        let init_string = ObjString::from("init");

        let class_name = "TestClass".into();
        let class_name_pointer = store.insert_string(class_name);
        let function = ObjFunction::default();
        let function_pointer = store.insert_function(function);
        let closure = ObjClosure {
            function: function_pointer.clone(),
            upvalues: Vec::new(),
        };
        let closure_pointer = store.insert_closure(closure);
        let mut methods = Table::default();
        methods.insert(init_string, closure_pointer.clone());
        let class = ObjClass {
            name: class_name_pointer.clone(),
            methods,
        };
        let class_pointer = store.insert_class(class);
        store
            .globals
            .insert("test_class".into(), class_pointer.clone().into());
        store.next_gc = 0;
        store.collect_garbage();
        assert!(store.string_store.contains_key(&class_name_pointer));
        assert!(store.function_store.contains_key(&function_pointer));
        assert!(store.closure_store.contains_key(&closure_pointer));
        assert!(store.class_store.contains_key(&class_pointer));
    }

    #[test]
    fn it_traces_closures() {
        let mut store = Store::default();
        let function = ObjFunction::default();
        let function_pointer = store.insert_function(function);
        let upvalue = ObjUpvalue::Open { location: 2 };
        let upvalue_pointer = store.insert_upvalue(upvalue);
        let closure = ObjClosure {
            function: function_pointer.clone(),
            upvalues: vec![upvalue_pointer.clone()],
        };
        let closure_pointer = store.insert_closure(closure);
        store
            .globals
            .insert("closure".into(), closure_pointer.clone().into());
        store.next_gc = 0;
        store.collect_garbage();
        assert!(store.function_store.contains_key(&function_pointer));
        assert!(store.upvalue_store.contains_key(&upvalue_pointer));
        assert!(store.closure_store.contains_key(&closure_pointer));
    }

    #[test]
    fn it_traces_instances() {
        let mut store = Store::default();
        let init_string = ObjString::from("init");
        let class_name = "TestClass".into();
        let class_name_pointer = store.insert_string(class_name);
        let function = ObjFunction::default();
        let function_pointer = store.insert_function(function);
        let closure = ObjClosure {
            function: function_pointer.clone(),
            upvalues: Vec::new(),
        };
        let closure_pointer = store.insert_closure(closure);
        let mut methods = Table::default();
        methods.insert(init_string, closure_pointer.clone());
        let class = ObjClass {
            name: class_name_pointer.clone(),
            methods,
        };
        let class_pointer = store.insert_class(class);
        let mut fields = Table::default();
        fields.insert("a".into(), RuntimeValue::Nil);
        let instance = ObjInstance {
            class: class_pointer.clone(),
            fields,
        };
        let instance_pointer = store.insert_instance(instance);
        store
            .globals
            .insert("test_instance".into(), instance_pointer.clone().into());
        store.next_gc = 0;
        store.collect_garbage();

        assert!(store.string_store.contains_key(&class_name_pointer));
        assert!(store.function_store.contains_key(&function_pointer));
        assert!(store.closure_store.contains_key(&closure_pointer));
        assert!(store.class_store.contains_key(&class_pointer));
        assert!(store.instance_store.contains_key(&instance_pointer));
    }
}
