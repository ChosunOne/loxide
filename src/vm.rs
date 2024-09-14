use std::{array, u8};

use crate::{
    compiler::Compiler,
    error::Error,
    object::{ObjClosure, ObjFunction, Object, ObjectStore},
    value::{runtime_pointer::ObjectReference, RuntimeReference, RuntimeValue},
};

const MAX_FRAMES: usize = 64;
const MAX_STACK_SIZE: usize = u8::MAX as usize * MAX_FRAMES;

pub struct VM {
    object_store: ObjectStore,
    value_stack: [RuntimeValue; MAX_STACK_SIZE],
    stack_top: usize,
}

impl VM {
    pub fn new() -> Self {
        Self {
            object_store: ObjectStore::default(),
            value_stack: array::from_fn(|_| RuntimeValue::default()),
            stack_top: 0,
        }
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
        let closure = self.new_closure();

        todo!();
    }

    fn new_closure(&mut self) -> RuntimeReference<ObjClosure> {
        todo!()
    }

    fn push_value(&mut self, value: impl Into<RuntimeValue>) {
        if self.stack_top == MAX_STACK_SIZE {
            panic!("Stack overflow.");
        }
        self.value_stack[self.stack_top] = value.into();
        self.stack_top += 1;
    }

    fn pop_value(&mut self) -> Option<RuntimeValue> {
        if self.stack_top == 0 {
            return None;
        }
        self.stack_top -= 1;
        Some(self.value_stack[self.stack_top])
    }

    fn peek_value(&mut self, distance: usize) -> Option<&mut RuntimeValue> {
        if self.value_stack.is_empty() || distance > self.value_stack.len() - 1 {
            return None;
        }
        let index = self.value_stack.len() - 1 - distance;
        self.value_stack.get_mut(index)
    }
}
