use std::{array, collections::HashMap};

use crate::{
    call_frame::CallFrame,
    chunk::OpCode,
    compiler::Compiler,
    error::Error,
    object::{
        object_store::GetPointer, ObjBoundMethod, ObjClass, ObjClosure, ObjFunction, ObjInstance,
        ObjNative, ObjString, ObjUpvalue, Object, ObjectStore,
    },
    value::{
        runtime_pointer::ObjectReference, ConstantValue, RuntimePointer, RuntimeReference,
        RuntimeValue,
    },
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
    globals: HashMap<String, RuntimeValue>,
}

impl Default for VM {
    fn default() -> Self {
        Self {
            object_store: ObjectStore::default(),
            value_stack: array::from_fn(|_| RuntimeValue::default()),
            frame_stack: array::from_fn(|_| todo!()),
            frame_stack_top: 0,
            value_stack_top: 0,
            globals: HashMap::default(),
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
        self.call(closure, 0)?;
        self.run()
    }

    fn runtime_error(&mut self, message: String) {
        todo!()
    }

    fn current_frame(&mut self) -> &mut CallFrame {
        &mut self.frame_stack[self.frame_stack_top]
    }

    fn get_closure_function(
        &mut self,
        closure: RuntimeReference<ObjClosure>,
    ) -> Option<RuntimePointer<'_, ObjFunction>> {
        let function_ref = self.get_pointer(closure).map(|x| x.function)?;
        self.get_pointer(function_ref)
    }

    fn read_byte(&mut self) -> Result<u8, Error> {
        let closure = self.current_frame().closure;
        let ip = self.current_frame().ip;
        let code = self
            .get_closure_function(closure)
            .ok_or(Error::Runtime)?
            .chunk
            .code[ip];

        self.current_frame().ip += 1;
        Ok(code)
    }

    fn read_short(&mut self) -> Result<u16, Error> {
        let byte_1 = self.read_byte()?;
        let byte_2 = self.read_byte()?;
        Ok((byte_1 as u16) << 8 | (byte_2 as u16))
    }

    fn read_constant(&mut self) -> Result<ConstantValue, Error> {
        let closure = self.current_frame().closure;
        let index = self.read_byte()?;
        Ok(self
            .get_closure_function(closure)
            .ok_or(Error::Runtime)?
            .chunk
            .constants[index as usize]
            .clone())
    }

    fn read_string(&mut self) -> Result<ObjString, Error> {
        let string = match self.read_constant()? {
            ConstantValue::String(s) => ObjString::from(s),
            _ => return Err(Error::Runtime),
        };
        Ok(string)
    }

    fn bind_method(
        &mut self,
        class: RuntimeReference<ObjClass>,
        name: ObjString,
    ) -> Result<(), Error> {
        todo!()
    }

    fn concatenate(&mut self) -> Result<(), Error> {
        todo!()
    }

    fn run(&mut self) -> Result<(), Error> {
        loop {
            let instruction = OpCode::from(self.read_byte()?);
            match instruction {
                OpCode::Constant => {
                    let constant = self.read_constant()?;
                    let runtime_value = match constant {
                        ConstantValue::Number(n) => RuntimeValue::Number(n),
                        ConstantValue::String(s) => {
                            let obj_string = ObjString { chars: s };
                            ObjectReference::from(self.object_store.insert(obj_string)).into()
                        }
                        ConstantValue::Function(f) => {
                            let obj_function = *f;
                            ObjectReference::from(self.object_store.insert(obj_function)).into()
                        }
                    };

                    self.push_value(runtime_value);
                }
                OpCode::Nil => self.push_value(RuntimeValue::Nil),
                OpCode::True => self.push_value(RuntimeValue::Bool(true)),
                OpCode::False => self.push_value(RuntimeValue::Bool(false)),
                OpCode::Pop => {
                    self.pop_value();
                }
                OpCode::GetLocal => {
                    let slot = self.current_frame().slots - self.read_byte()? as usize;
                    let value = *self.peek_value(slot).ok_or(Error::Runtime)?;
                    self.push_value(value);
                }
                OpCode::SetLocal => {
                    let slot = self.current_frame().slots - self.read_byte()? as usize;
                    let value = *self.peek_value(0).ok_or(Error::Runtime)?;
                    *self.peek_value(slot).ok_or(Error::Runtime)? = value;
                }
                OpCode::GetGlobal => {
                    let name = self.read_string()?;
                    let value = match self.globals.get(&name.chars) {
                        Some(v) => *v,
                        None => {
                            self.runtime_error("Undefined variable {name}".into());
                            return Err(Error::Runtime);
                        }
                    };
                    self.push_value(value);
                }
                OpCode::SetGlobal => {
                    let name = self.read_string()?;
                    if self.globals.contains_key(&name.chars) {
                        self.runtime_error("Undefined variable '{name}'.".into());
                        return Err(Error::Runtime);
                    }
                    let value = *self.peek_value(0).ok_or(Error::Runtime)?;
                    self.globals.insert(name.chars, value);
                }
                OpCode::DefineGlobal => {
                    let name = self.read_string()?;
                    let value = *self.peek_value(0).ok_or(Error::Runtime)?;
                    self.globals.insert(name.chars, value);
                }
                OpCode::GetUpvalue => {
                    let slot = self.read_byte()? as usize;
                    let upvalue = {
                        let closure_ref = self.current_frame().closure;
                        let closure = self.get_pointer(closure_ref).ok_or(Error::Runtime)?;
                        let upvalue_ref = closure.upvalues[slot];
                        self.get_pointer(upvalue_ref).ok_or(Error::Runtime)?
                    };
                    let location = upvalue.location;
                    self.push_value(location);
                }
                OpCode::SetUpvalue => {
                    let slot = self.read_byte()? as usize;
                    let value = *self.peek_value(0).ok_or(Error::Runtime)?;
                    let mut upvalue = {
                        let closure_ref = self.current_frame().closure;
                        let closure = self.get_pointer(closure_ref).ok_or(Error::Runtime)?;
                        let upvalue_ref = closure.upvalues[slot];
                        self.get_pointer(upvalue_ref).ok_or(Error::Runtime)?
                    };
                    upvalue.location = value;
                }
                OpCode::GetProperty => {
                    let name = self.read_string()?;
                    let instance = {
                        let instance_ref = self
                            .peek_typed::<RuntimeReference<ObjInstance>>(0)
                            .ok_or_else(|| {
                                self.runtime_error("Only instances have fields.".into());
                                Error::Runtime
                            })?;
                        self.get_pointer(instance_ref).ok_or(Error::Runtime)?
                    };
                    if let Some(&v) = instance.fields.get(&name.chars) {
                        self.pop_value(); // Instance
                        self.push_value(v);
                        continue;
                    }

                    let class = instance.class;
                    self.bind_method(class, name)?;
                }
                OpCode::SetProperty => {
                    let instance_ref = self
                        .peek_typed::<RuntimeReference<ObjInstance>>(1)
                        .ok_or_else(|| {
                            self.runtime_error("Only instances have fields.".into());
                            Error::Runtime
                        })?;
                    let name = self.read_string()?;
                    let value = *self.peek_value(0).ok_or(Error::Runtime)?;
                    let mut instance = self.get_pointer(instance_ref).ok_or(Error::Runtime)?;
                    instance.fields.insert(name.chars, value);
                    let value = self.pop_value().ok_or(Error::Runtime)?;
                    self.pop_value(); // Instance
                    self.push_value(value);
                }
                OpCode::GetSuper => {
                    let name = self.read_string()?;
                    let superclass = match self.pop_value().ok_or(Error::Runtime)? {
                        RuntimeValue::Object(ObjectReference::Class(o)) => o,
                        _ => return Err(Error::Runtime),
                    };
                    self.bind_method(superclass, name)?;
                }
                OpCode::Equal => {
                    let a = self.pop_value().ok_or(Error::Runtime)?;
                    let b = self.pop_value().ok_or(Error::Runtime)?;
                    self.push_value(a == b);
                }
                OpCode::Greater => {
                    if self.peek_typed::<f64>(0).is_none() || self.peek_typed::<f64>(1).is_none() {
                        self.runtime_error("Operands must be numbers".into());
                        return Err(Error::Runtime);
                    }
                    let b = self.pop_typed::<f64>().ok_or(Error::Runtime)?;
                    let a = self.pop_typed::<f64>().ok_or(Error::Runtime)?;
                    self.push_value(a > b);
                }
                OpCode::Less => {
                    if self.peek_typed::<f64>(0).is_none() || self.peek_typed::<f64>(1).is_none() {
                        self.runtime_error("Operands must be numbers".into());
                        return Err(Error::Runtime);
                    }
                    let b = self.pop_typed::<f64>().ok_or(Error::Runtime)?;
                    let a = self.pop_typed::<f64>().ok_or(Error::Runtime)?;
                    self.push_value(a < b);
                }
                OpCode::Add => {
                    if self.peek_typed::<RuntimeReference<ObjString>>(0).is_some()
                        && self.peek_typed::<RuntimeReference<ObjString>>(1).is_some()
                    {
                        self.concatenate()?;
                        continue;
                    }

                    if self.peek_typed::<f64>(0).is_none() || self.peek_typed::<f64>(1).is_none() {
                        self.runtime_error("Operands must be two numbers or two strings.".into());
                        return Err(Error::Runtime);
                    }
                    let b = self.pop_typed::<f64>().ok_or(Error::Runtime)?;
                    let a = self.pop_typed::<f64>().ok_or(Error::Runtime)?;
                    self.push_value(a + b);
                }
                OpCode::Subtract => {
                    if self.peek_typed::<f64>(0).is_none() || self.peek_typed::<f64>(1).is_none() {
                        self.runtime_error("Operands must be numbers".into());
                        return Err(Error::Runtime);
                    }
                    let b = self.pop_typed::<f64>().ok_or(Error::Runtime)?;
                    let a = self.pop_typed::<f64>().ok_or(Error::Runtime)?;
                    self.push_value(a - b);
                }
                OpCode::Multiply => {
                    if self.peek_typed::<f64>(0).is_none() || self.peek_typed::<f64>(1).is_none() {
                        self.runtime_error("Operands must be numbers".into());
                        return Err(Error::Runtime);
                    }
                    let b = self.pop_typed::<f64>().ok_or(Error::Runtime)?;
                    let a = self.pop_typed::<f64>().ok_or(Error::Runtime)?;
                    self.push_value(a * b);
                }
                OpCode::Divide => {
                    if self.peek_typed::<f64>(0).is_none() || self.peek_typed::<f64>(1).is_none() {
                        self.runtime_error("Operands must be numbers".into());
                        return Err(Error::Runtime);
                    }
                    let b = self.pop_typed::<f64>().ok_or(Error::Runtime)?;
                    let a = self.pop_typed::<f64>().ok_or(Error::Runtime)?;
                    self.push_value(a / b);
                }
                OpCode::Not => {
                    let value = self.pop_value().ok_or(Error::Runtime)?;
                    self.push_value(value.is_falsey());
                }
                OpCode::Negate => {
                    let Some(value) = self.pop_typed::<f64>() else {
                        self.runtime_error("Operand must be a number.".into());
                        return Err(Error::Runtime);
                    };
                    self.push_value(-value);
                }
                OpCode::Print => {
                    let value = self.pop_value().ok_or(Error::Runtime)?;
                    match value {
                        RuntimeValue::Bool(b) => println!("{b}"),
                        RuntimeValue::Number(n) => println!("{n}"),
                        RuntimeValue::Object(o) => match o {
                            ObjectReference::BoundMethod(runtime_reference) => {
                                let method_ref = self
                                    .get_pointer(runtime_reference)
                                    .ok_or(Error::Runtime)?
                                    .method;
                                let function_ptr = self
                                    .get_closure_function(method_ref)
                                    .ok_or(Error::Runtime)?;
                                println!("{}", *function_ptr);
                            }
                            ObjectReference::Class(runtime_reference) => {
                                let name_ref = self
                                    .get_pointer(runtime_reference)
                                    .ok_or(Error::Runtime)?
                                    .name;
                                let name_ptr = self.get_pointer(name_ref).ok_or(Error::Runtime)?;
                                println!("{}", *name_ptr);
                            }
                            ObjectReference::Closure(runtime_reference) => {
                                let function_ptr = self
                                    .get_closure_function(runtime_reference)
                                    .ok_or(Error::Runtime)?;
                                println!("{}", *function_ptr);
                            }
                            ObjectReference::Function(runtime_reference) => {
                                let function_ptr =
                                    self.get_pointer(runtime_reference).ok_or(Error::Runtime)?;
                                println!("{}", *function_ptr);
                            }
                            ObjectReference::Instance(runtime_reference) => {
                                let class_ref = self
                                    .get_pointer(runtime_reference)
                                    .ok_or(Error::Runtime)?
                                    .class;
                                let name_ref =
                                    self.get_pointer(class_ref).ok_or(Error::Runtime)?.name;
                                let name = self.get_pointer(name_ref).ok_or(Error::Runtime)?;
                                println!("{} instance", *name);
                            }
                            ObjectReference::Native(runtime_reference) => {
                                let native_ptr =
                                    self.get_pointer(runtime_reference).ok_or(Error::Runtime)?;
                                println!("{}", *native_ptr);
                            }
                            ObjectReference::String(runtime_reference) => {
                                let string_ptr =
                                    self.get_pointer(runtime_reference).ok_or(Error::Runtime)?;
                                println!("{}", *string_ptr);
                            }
                            ObjectReference::Upvalue(runtime_reference) => {
                                let upvalue_ptr =
                                    self.get_pointer(runtime_reference).ok_or(Error::Runtime)?;
                                println!("{}", *upvalue_ptr);
                            }
                        },
                        RuntimeValue::Nil => println!("nil"),
                    }
                }
                OpCode::Jump => todo!(),
                OpCode::JumpIfFalse => todo!(),
                OpCode::Loop => todo!(),
                OpCode::Call => todo!(),
                OpCode::Invoke => todo!(),
                OpCode::SuperInvoke => todo!(),
                OpCode::Closure => todo!(),
                OpCode::CloseUpvalue => todo!(),
                OpCode::Return => todo!(),
                OpCode::Class => todo!(),
                OpCode::Inherit => todo!(),
                OpCode::Method => todo!(),
                OpCode::Unknown => return Err(Error::Runtime),
            }
        }
    }

    fn call(
        &mut self,
        closure: RuntimeReference<ObjClosure>,
        arg_count: usize,
    ) -> Result<(), Error> {
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
            return Err(Error::Runtime);
        }
        if self.frame_stack_top == MAX_FRAMES {
            self.runtime_error("Stack overflow.".into());
            return Err(Error::Runtime);
        }
        let frame = &mut self.frame_stack[self.frame_stack_top];
        frame.closure = closure;
        frame.slots = arg_count;
        self.frame_stack_top += 1;
        Ok(())
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

    fn peek_typed<T: TryFrom<RuntimeValue>>(&mut self, distance: usize) -> Option<T> {
        (*self.peek_value(distance)?).try_into().ok()
    }

    fn pop_typed<T: TryFrom<RuntimeValue>>(&mut self) -> Option<T> {
        self.pop_value()?.try_into().ok()
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

impl GetPointer<ObjUpvalue> for VM {
    fn get_pointer(
        &mut self,
        key: RuntimeReference<ObjUpvalue>,
    ) -> Option<RuntimePointer<ObjUpvalue>> {
        self.object_store.get_pointer(key)
    }
}
