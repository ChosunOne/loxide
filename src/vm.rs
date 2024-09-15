use std::array;

use crate::{
    call_frame::CallFrame,
    compiler::Compiler,
    error::Error,
    object::{
        object_store::GetPointer, ObjBoundMethod, ObjClass, ObjClosure, ObjFunction, ObjInstance,
        ObjNative, ObjString, ObjUpvalue, Object, ObjectStore,
    },
    value::{runtime_pointer::ObjectReference, RuntimePointer, RuntimeReference, RuntimeValue},
};

const MAX_FRAMES: usize = 64;
const MAX_STACK_SIZE: usize = u8::MAX as usize * MAX_FRAMES;

#[derive(Debug)]
pub struct VM {
    object_store: ObjectStore,
    value_stack: [RuntimeValue; MAX_STACK_SIZE],
    frame_stack: [CallFrame; MAX_FRAMES],
    value_stack_top: usize,
    frame_stack_top: usize,
}

impl Default for VM {
    fn default() -> Self {
        Self {
            object_store: ObjectStore::default(),
            value_stack: array::from_fn(|_| RuntimeValue::default()),
            frame_stack: array::from_fn(|_| todo!()),
            frame_stack_top: 0,
            value_stack_top: 0,
        }
    }
}

impl VM {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn interpret(&mut self, source: &str) -> Result<(), Error> {
        #[cfg(feature = "debug")]
        println!("========== CODE ==========");
        let compiler = Compiler::new(source.into());

        let function = compiler.compile()?;

        #[cfg(feature = "debug")]
        println!("{}", function.chunk);

        let function = Box::pin(Object::Function(function));
        let function_ref = ObjectReference::from(RuntimeReference::<ObjFunction>::from(&*function));
        self.push_value(function_ref);
        self.object_store.insert_pinned(function);
        let closure = self.new_closure(function_ref.try_into()?);
        self.pop_value();
        self.push_value(closure);
        self.call(closure, 0);
        self.run()
    }

    fn runtime_error(&mut self, message: String) {
        todo!()
    }

    fn run(&mut self) -> Result<(), Error> {
        todo!()
    }

    fn call(&mut self, closure: RuntimeReference<ObjClosure>, arg_count: usize) -> bool {
        let arity = {
            let function_ref = self
                .get_pointer(closure)
                .expect("Failed to get pointer")
                .function;
            self.get_pointer(function_ref)
                .expect("Failed to get function pointer")
                .arity
        };
        if arg_count != arity {
            self.runtime_error(format!(
                "Expected {} arguments but got {}.",
                arity, arg_count
            ));
            return false;
        }
        todo!()
    }

    fn new_closure(
        &mut self,
        function: RuntimeReference<ObjFunction>,
    ) -> RuntimeReference<ObjClosure> {
        let function_ptr = self
            .object_store
            .get_pointer(function)
            .expect("Failed to get function pointer");
        let upvalues = Vec::with_capacity(function_ptr.upvalue_count);
        let closure = ObjClosure { function, upvalues };
        (&self.object_store.insert(closure)).into()
    }

    fn push_value(&mut self, value: impl Into<RuntimeValue>) {
        if self.value_stack_top == MAX_STACK_SIZE {
            panic!("Stack overflow.");
        }
        self.value_stack[self.value_stack_top] = value.into();
        self.value_stack_top += 1;
    }

    fn pop_value(&mut self) -> Option<RuntimeValue> {
        if self.value_stack_top == 0 {
            return None;
        }
        self.value_stack_top -= 1;
        Some(self.value_stack[self.value_stack_top])
    }

    fn peek_value(&mut self, distance: usize) -> Option<&mut RuntimeValue> {
        if self.value_stack.is_empty() || distance > self.value_stack.len() - 1 {
            return None;
        }
        let index = self.value_stack.len() - 1 - distance;
        self.value_stack.get_mut(index)
    }
}

impl GetPointer<ObjBoundMethod> for VM {
    fn get_pointer(
        &mut self,
        key: RuntimeReference<ObjBoundMethod>,
    ) -> Option<RuntimePointer<ObjBoundMethod>> {
        self.object_store.get_pointer(key)
    }
}

impl GetPointer<ObjClass> for VM {
    fn get_pointer(&mut self, key: RuntimeReference<ObjClass>) -> Option<RuntimePointer<ObjClass>> {
        self.object_store.get_pointer(key)
    }
}

impl GetPointer<ObjClosure> for VM {
    fn get_pointer(
        &mut self,
        key: RuntimeReference<ObjClosure>,
    ) -> Option<RuntimePointer<ObjClosure>> {
        self.object_store.get_pointer(key)
    }
}

impl GetPointer<ObjFunction> for VM {
    fn get_pointer(
        &mut self,
        key: RuntimeReference<ObjFunction>,
    ) -> Option<RuntimePointer<ObjFunction>> {
        self.object_store.get_pointer(key)
    }
}

impl GetPointer<ObjInstance> for VM {
    fn get_pointer(
        &mut self,
        key: RuntimeReference<ObjInstance>,
    ) -> Option<RuntimePointer<ObjInstance>> {
        self.object_store.get_pointer(key)
    }
}

impl GetPointer<ObjNative> for VM {
    fn get_pointer(
        &mut self,
        key: RuntimeReference<ObjNative>,
    ) -> Option<RuntimePointer<ObjNative>> {
        self.object_store.get_pointer(key)
    }
}

impl GetPointer<ObjString> for VM {
    fn get_pointer(
        &mut self,
        key: RuntimeReference<ObjString>,
    ) -> Option<RuntimePointer<ObjString>> {
        self.object_store.get_pointer(key)
    }
}
