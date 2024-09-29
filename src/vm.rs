use std::{
    cell::RefCell,
    collections::BTreeMap,
    io::{Stderr, Stdout, Write},
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    call_frame::CallFrame,
    chunk::OpCode,
    compiler::Compiler,
    error::Error,
    object::{
        obj_native::NativeFn, store::MAX_STACK_SIZE, ObjBoundMethod, ObjClass, ObjClosure,
        ObjFunction, ObjInstance, ObjNative, ObjString, ObjUpvalue, Pointer, Store,
    },
    table::Table,
    value::{ConstantValue, RuntimeValue},
};

pub const MAX_FRAMES: usize = 64;

fn clock_native(_args: Vec<RuntimeValue>) -> RuntimeValue {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("IVME: Failed to get system time")
        .as_secs_f64()
        .into()
}

#[derive(Debug)]
pub struct VM<Out: Write = Stdout, EOut: Write = Stderr> {
    store: Store,
    out: Out,
    e_out: EOut,
    init_string: ObjString,
}

impl<Out: Write, EOut: Write> VM<Out, EOut> {
    pub fn new(out: Out, e_out: EOut) -> Self {
        let store = Store::default();
        let mut vm = Self {
            store,
            out,
            e_out,
            init_string: "init".into(),
        };

        vm.define_native("clock".into(), clock_native);

        vm
    }

    pub fn interpret(&mut self, source: &str) -> Result<(), Error> {
        #[cfg(feature = "debug")]
        println!("========== CODE ==========");

        let compiler = Compiler::new(source.into());

        let function = compiler.compile()?;
        #[cfg(feature = "debug")]
        {
            println!("== {} ==", function);
            println!("{}", function.chunk);
        }

        let function = Rc::new(RefCell::new(function));
        let function_ref = self.store.insert_function_pointer(function);
        self.push_value(function_ref.clone());
        let closure = self.new_closure(function_ref);
        self.pop_value();
        self.push_value(closure.clone());
        self.call(closure, 0)?;
        self.run()?;
        self.pop_value();
        Ok(())
    }

    fn define_native(&mut self, name: ObjString, function: NativeFn) {
        let native_pointer = self.new_native(function).into();
        self.store.globals.insert(name, native_pointer);
    }

    fn println(&mut self, string: impl Into<String>) -> Result<(), Error> {
        let string: String = string.into() + "\n";
        self.out
            .write_all(string.as_bytes())
            .expect("IVME: Failed to write data");
        self.out.flush().expect("IVME: Failed to flush data");
        Ok(())
    }

    fn eprint(&mut self, string: impl Into<String>) -> Result<(), Error> {
        let string: String = string.into();
        self.e_out
            .write_all(string.as_bytes())
            .expect("IVME: Failed to write data");
        self.e_out.flush().expect("IVME: Failed to flush data");
        Ok(())
    }

    fn reset_stack(&mut self) {
        self.store.value_stack_top = 0;
        self.store.frame_stack_top = 0;
        self.store.open_upvalues = BTreeMap::default();
    }

    fn runtime_error(&mut self, message: String) {
        self.eprint(message).expect("IVME: Failed to print error");

        while let Some(frame) = self.pop_frame() {
            let function = frame
                .closure
                .expect("IVME: Failed to get frame closure")
                .borrow()
                .function
                .clone();
            let line = function.borrow().chunk.lines[frame.ip];
            self.eprint(format!("[line {line}] in "))
                .expect("IVME: Failed to print error");
            if let Some(name) = function.borrow().name.as_ref() {
                self.eprint(format!("{name}\n"))
                    .expect("IVME: Failed to print error");
            } else {
                self.eprint("script\n")
                    .expect("IVME: Failed to print error");
            };
        }

        self.reset_stack();
    }

    fn current_frame(&self) -> &CallFrame {
        &self.store.frame_stack[self.store.frame_stack_top - 1]
    }

    fn current_frame_mut(&mut self) -> &mut CallFrame {
        &mut self.store.frame_stack[self.store.frame_stack_top - 1]
    }

    fn current_closure(&self) -> Pointer<ObjClosure> {
        self.current_frame()
            .closure
            .clone()
            .expect("IVME: Failed to get currently executing closure.")
    }

    fn read_byte(&mut self) -> u8 {
        let closure = self.current_closure();
        let ip = self.current_frame().ip;
        let code = closure.borrow().function.borrow().chunk.code[ip];
        self.current_frame_mut().ip += 1;
        code
    }

    fn read_short(&mut self) -> u16 {
        let byte_1 = self.read_byte();
        let byte_2 = self.read_byte();
        (byte_1 as u16) << 8 | (byte_2 as u16)
    }

    fn read_constant(&mut self) -> Rc<ConstantValue> {
        let closure = self.current_closure();
        let index = self.read_byte();
        let constant = closure.borrow().function.borrow().chunk.constants[index as usize].clone();
        constant
    }

    fn bind_method(&mut self, class: Pointer<ObjClass>, name: &ObjString) -> Result<(), Error> {
        let class = class.borrow();
        let Some(method) = class.methods.get(name) else {
            self.runtime_error(format!("Undefined property '{}'", name.chars));
            return Err(Error::Runtime);
        };

        let receiver = self.peek_value(0).clone();
        let bound = self.new_bound_method(receiver, method.clone());
        self.pop_value();
        self.push_value(bound);
        Ok(())
    }

    fn capture_upvalue(&mut self, index: usize) -> Result<Pointer<ObjUpvalue>, Error> {
        let absolute_stack_index = self.current_frame().start_stack_index + index;
        if let Some(upvalue) = self.store.open_upvalues.get(&absolute_stack_index) {
            return Ok(upvalue.clone());
        }

        let upvalue = ObjUpvalue::Open {
            location: absolute_stack_index,
        };
        let upvalue_ptr = self.store.insert_upvalue(upvalue);

        self.store
            .open_upvalues
            .insert(absolute_stack_index, upvalue_ptr.clone());
        Ok(upvalue_ptr)
    }

    fn close_upvalues(&mut self, last_stack_index: usize) -> Result<(), Error> {
        let abs_last_stack_index = self.current_frame().start_stack_index + last_stack_index;
        let mut closed_upvalues = Vec::new();
        for (&abs_stack_index, open_upvalue) in self.store.open_upvalues.iter_mut().rev() {
            if abs_stack_index < abs_last_stack_index {
                break;
            }
            let referenced_value = self.store.value_stack[abs_stack_index].clone();
            *open_upvalue.borrow_mut() = ObjUpvalue::Closed {
                value: referenced_value,
            };
            closed_upvalues.push(abs_stack_index);
        }
        for closed_upvalue in closed_upvalues {
            self.store.open_upvalues.remove(&closed_upvalue);
        }
        Ok(())
    }

    fn define_method(&mut self, name: &ObjString) -> Result<(), Error> {
        let method = self.peek_typed::<Pointer<ObjClosure>>(0)?;
        let class = self.peek_typed::<Pointer<ObjClass>>(1)?;
        class.borrow_mut().methods.insert(name.clone(), method);
        self.pop_value();
        Ok(())
    }

    fn concatenate(&mut self) -> Result<(), Error> {
        let b = self.peek_typed::<Pointer<ObjString>>(0)?;
        let a = self.peek_typed::<Pointer<ObjString>>(1)?;
        let result = a.borrow().chars.clone() + &b.borrow().chars;
        let new_string = self.store.insert_string(result.into());
        self.pop_value();
        self.pop_value();
        self.push_value(new_string);
        Ok(())
    }

    fn invoke(&mut self, method_name: &ObjString, arg_count: usize) -> Result<(), Error> {
        let receiver = self
            .peek_typed::<Pointer<ObjInstance>>(arg_count)
            .expect("IVME: Failed to get instance.");
        let instance_fields = &receiver.borrow().fields;
        if let Some(value) = instance_fields.get(method_name) {
            self.store.value_stack[self.store.value_stack_top - arg_count - 1] = value.clone();
            return self.call_value(value.clone(), arg_count);
        }
        let class = receiver.borrow().class.clone();
        self.invoke_from_class(class, method_name, arg_count)
    }

    fn invoke_from_class(
        &mut self,
        class: Pointer<ObjClass>,
        method_name: &ObjString,
        arg_count: usize,
    ) -> Result<(), Error> {
        let class = class.borrow();
        let Some(method) = class.methods.get(method_name) else {
            self.runtime_error(format!("Undefined property '{method_name}'.\n"));
            return Err(Error::Runtime);
        };
        self.call(method.clone(), arg_count)
    }

    fn call_value(&mut self, callee: RuntimeValue, arg_count: usize) -> Result<(), Error> {
        match callee {
            RuntimeValue::BoundMethod(bm) => {
                *self.peek_value(arg_count) = bm.borrow().receiver.clone();
                let method = bm.borrow().method.clone();
                self.call(method, arg_count)
            }
            RuntimeValue::Class(class) => {
                let instance = self.new_instance(class.clone());
                *self.peek_value(arg_count) = instance.into();
                let class = class.borrow();
                if let Some(initializer) = class.methods.get(&self.init_string) {
                    self.call(initializer.clone(), arg_count)?;
                } else if arg_count != 0 {
                    self.runtime_error(format!("Expected 0 arguments but got {arg_count}.\n"));
                    return Err(Error::Runtime);
                }
                Ok(())
            }
            RuntimeValue::Closure(closure) => self.call(closure, arg_count),
            RuntimeValue::Native(native) => {
                let native = native.borrow();
                let args = (self.store.value_stack
                    [self.store.value_stack_top - arg_count..self.store.value_stack_top])
                    .to_vec();
                let result = (native.function)(args);
                self.store.value_stack_top -= arg_count + 1;
                self.push_value(result);
                Ok(())
            }

            _ => {
                self.runtime_error("Can only call functions and classes.\n".into());
                Err(Error::Runtime)
            }
        }
    }

    fn frame_slot_to_peek_distance(&self, slot: usize) -> usize {
        let slot_distance =
            self.store.value_stack_top - 1 - (self.current_frame().start_stack_index + slot);
        slot_distance
    }

    fn run(&mut self) -> Result<(), Error> {
        loop {
            let instruction = OpCode::from(self.read_byte());
            #[cfg(feature = "debug")]
            {
                println!();
                for i in 0..self.store.value_stack_top {
                    print!("[ {} ]", self.store.value_stack[i]);
                }
                println!();
                println!("{instruction}");
            }
            match instruction {
                OpCode::Constant => {
                    let constant = &*self.read_constant();
                    let runtime_value = match constant {
                        ConstantValue::Number(n) => RuntimeValue::Number(*n),
                        ConstantValue::String(s) => {
                            let obj_string = s.clone();
                            self.store.insert_string(obj_string).into()
                        }
                        ConstantValue::Function(f) => {
                            let obj_function = *f.clone();
                            self.store.insert_function(obj_function).into()
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
                    let slot = self.read_byte() as usize;
                    let slot_distance = self.frame_slot_to_peek_distance(slot);

                    let value = self.peek_value(slot_distance).clone();
                    self.push_value(value);
                }
                OpCode::SetLocal => {
                    let slot = self.read_byte() as usize;
                    let slot_distance = self.frame_slot_to_peek_distance(slot);
                    let value = self.peek_value(0).clone();
                    *self.peek_value(slot_distance) = value;
                }
                OpCode::GetGlobal => {
                    let ConstantValue::String(name) = &*self.read_constant() else {
                        panic!("IVME: Unexpected constant value.")
                    };
                    let value = match self.store.globals.get(name) {
                        Some(v) => v.clone(),
                        None => {
                            self.runtime_error(format!("Undefined variable '{name}'.\n"));
                            return Err(Error::Runtime);
                        }
                    };
                    self.push_value(value);
                }
                OpCode::SetGlobal => {
                    let ConstantValue::String(name) = &*self.read_constant() else {
                        panic!("IVME: Unexpected constant value.")
                    };
                    let value = self.peek_value(0).clone();
                    if self.store.globals.insert(name.clone(), value) {
                        self.store.globals.remove(name);
                        self.runtime_error(format!("Undefined variable '{name}'.\n"));
                        return Err(Error::Runtime);
                    }
                }
                OpCode::DefineGlobal => {
                    let ConstantValue::String(name) = &*self.read_constant() else {
                        panic!("IVME: Unexpected constant value.")
                    };
                    let value = self.pop_value().clone();
                    self.store.globals.insert(name.clone(), value);
                }
                OpCode::GetUpvalue => {
                    let slot = self.read_byte() as usize;
                    let location = {
                        let closure = self.current_closure();
                        let upvalue = closure.borrow().upvalues[slot].clone();
                        let upvalue_deref = upvalue.borrow();
                        match &*upvalue_deref {
                            ObjUpvalue::Open { location } => *location,
                            ObjUpvalue::Closed { value } => {
                                self.push_value(value.clone());
                                continue;
                            }
                        }
                    };
                    let value = self.store.value_stack[location].clone();
                    self.push_value(value);
                }
                OpCode::SetUpvalue => {
                    let slot = self.read_byte() as usize;
                    let closure = self.current_closure();
                    let open_upvalue = self.store.insert_upvalue(ObjUpvalue::Open {
                        location: self.store.value_stack_top - 1,
                    });
                    let mut closure = closure.borrow_mut();
                    closure.upvalues[slot] = open_upvalue;
                }
                OpCode::GetProperty => {
                    let ConstantValue::String(name) = &*self.read_constant() else {
                        panic!("IVME: Unexpected constant value.")
                    };
                    let instance = {
                        let Ok(instance_ref) = self.peek_typed::<Pointer<ObjInstance>>(0) else {
                            self.runtime_error("Only instances have fields.\n".into());
                            return Err(Error::Runtime);
                        };
                        instance_ref
                    };
                    let instance = instance.borrow();
                    if let Some(v) = instance.fields.get(name) {
                        self.pop_value(); // Instance
                        self.push_value(v.clone());
                        continue;
                    }

                    let class = instance.class.clone();
                    self.bind_method(class, name)?;
                }
                OpCode::SetProperty => {
                    let Ok(instance) = self.peek_typed::<Pointer<ObjInstance>>(1) else {
                        self.runtime_error("Only instances have fields.\n".into());
                        return Err(Error::Runtime);
                    };
                    let ConstantValue::String(name) = &*self.read_constant() else {
                        panic!("IVME: Unexpected constant value.")
                    };
                    let value = self.peek_value(0);
                    instance
                        .borrow_mut()
                        .fields
                        .insert(name.clone(), value.clone());
                    let value = self.pop_value();
                    self.pop_value(); // Instance
                    self.push_value(value);
                }
                OpCode::GetSuper => {
                    let ConstantValue::String(name) = &*self.read_constant() else {
                        panic!("IVME: Unexpected constant value.")
                    };
                    let superclass = match self.pop_value() {
                        RuntimeValue::Class(o) => o,
                        _ => return Err(Error::Runtime),
                    };
                    self.bind_method(superclass, name)?;
                }
                OpCode::Equal => {
                    let a = self.pop_value();
                    let b = self.pop_value();
                    self.push_value(a == b);
                }
                OpCode::Greater => {
                    if self.peek_typed::<f64>(0).is_err() || self.peek_typed::<f64>(1).is_err() {
                        self.runtime_error("Operands must be numbers.\n".into());
                        return Err(Error::Runtime);
                    }
                    let b = self.pop_typed::<f64>();
                    let a = self.pop_typed::<f64>();
                    self.push_value(a > b);
                }
                OpCode::Less => {
                    if self.peek_typed::<f64>(0).is_err() || self.peek_typed::<f64>(1).is_err() {
                        self.runtime_error("Operands must be numbers.\n".into());
                        return Err(Error::Runtime);
                    }
                    let b = self.pop_typed::<f64>();
                    let a = self.pop_typed::<f64>();
                    self.push_value(a < b);
                }
                OpCode::Add => {
                    if self.peek_typed::<Pointer<ObjString>>(0).is_ok()
                        && self.peek_typed::<Pointer<ObjString>>(1).is_ok()
                    {
                        self.concatenate()?;
                        continue;
                    }

                    if self.peek_typed::<f64>(0).is_err() || self.peek_typed::<f64>(1).is_err() {
                        self.runtime_error("Operands must be two numbers or two strings.\n".into());
                        return Err(Error::Runtime);
                    }
                    let b = self.pop_typed::<f64>();
                    let a = self.pop_typed::<f64>();
                    self.push_value(a + b);
                }
                OpCode::Subtract => {
                    if self.peek_typed::<f64>(0).is_err() || self.peek_typed::<f64>(1).is_err() {
                        self.runtime_error("Operands must be numbers.\n".into());
                        return Err(Error::Runtime);
                    }
                    let b = self.pop_typed::<f64>();
                    let a = self.pop_typed::<f64>();
                    self.push_value(a - b);
                }
                OpCode::Multiply => {
                    if self.peek_typed::<f64>(0).is_err() || self.peek_typed::<f64>(1).is_err() {
                        self.runtime_error("Operands must be numbers.\n".into());
                        return Err(Error::Runtime);
                    }
                    let b = self.pop_typed::<f64>();
                    let a = self.pop_typed::<f64>();
                    self.push_value(a * b);
                }
                OpCode::Divide => {
                    if self.peek_typed::<f64>(0).is_err() || self.peek_typed::<f64>(1).is_err() {
                        self.runtime_error("Operands must be numbers.\n".into());
                        return Err(Error::Runtime);
                    }
                    let b = self.pop_typed::<f64>();
                    let a = self.pop_typed::<f64>();
                    self.push_value(a / b);
                }
                OpCode::Not => {
                    let value = self.pop_value();
                    self.push_value(value.is_falsey());
                }
                OpCode::Negate => {
                    if self.peek_typed::<f64>(0).is_err() {
                        self.runtime_error("Operand must be a number.\n".into());
                        return Err(Error::Runtime);
                    }
                    let value = self.pop_typed::<f64>();
                    self.push_value(-value);
                }
                OpCode::Print => {
                    let value = self.pop_value();
                    match value {
                        RuntimeValue::Bool(b) => self.println(format!("{b}"))?,
                        RuntimeValue::Number(n) => {
                            if n.fract() == 0.0 {
                                self.println(format!("{n}"))?;
                            } else {
                                self.println(format!("{n:.6}"))?;
                            }
                        }
                        RuntimeValue::BoundMethod(bm) => {
                            self.println(format!("{bm}"))?;
                        }
                        RuntimeValue::Class(class) => {
                            self.println(format!("{class}"))?;
                        }
                        RuntimeValue::Closure(closure) => {
                            self.println(format!("{closure}"))?;
                        }
                        RuntimeValue::Function(function) => {
                            self.println(format!("{function}"))?;
                        }
                        RuntimeValue::Instance(instance) => {
                            self.println(format!("{instance}"))?;
                        }
                        RuntimeValue::Native(native) => {
                            self.println(format!("{native}"))?;
                        }
                        RuntimeValue::String(string) => {
                            self.println(format!("{string}"))?;
                        }
                        RuntimeValue::Nil => self.println("nil")?,
                        RuntimeValue::Upvalue(upvalue) => {
                            self.println(format!("{upvalue}"))?;
                        }
                    }
                }
                OpCode::Jump => {
                    let offset = self.read_short() as usize;
                    self.current_frame_mut().ip += offset;
                }
                OpCode::JumpIfFalse => {
                    let offset = self.read_short() as usize;
                    if self.peek_value(0).is_falsey() {
                        self.current_frame_mut().ip += offset;
                    }
                }
                OpCode::Loop => {
                    let offset = self.read_short() as usize;
                    self.current_frame_mut().ip -= offset;
                }
                OpCode::Call => {
                    let arg_count = self.read_byte() as usize;
                    let callee = self.peek_value(arg_count).clone();
                    self.call_value(callee, arg_count)?;
                }
                OpCode::Invoke => {
                    let ConstantValue::String(method_name) = &*self.read_constant() else {
                        panic!("IVME: Unexpected constant value.")
                    };
                    let arg_count = self.read_byte() as usize;
                    self.invoke(method_name, arg_count)?;
                }
                OpCode::SuperInvoke => {
                    let ConstantValue::String(method_name) = &*self.read_constant() else {
                        panic!("IVME: Unexpected constant value.")
                    };
                    let arg_count = self.read_byte() as usize;
                    let class = self.pop_typed::<Pointer<ObjClass>>();
                    self.invoke_from_class(class, method_name, arg_count)?;
                }
                OpCode::Closure => {
                    let ConstantValue::Function(function) = &*self.read_constant().clone() else {
                        return Err(Error::Runtime);
                    };
                    let upvalue_count = function.upvalue_count;
                    let function = self.store.insert_function(*function.clone());
                    let closure = self.new_closure(function);
                    self.push_value(closure.clone());
                    let current_closure = self.current_closure();
                    for _ in 0..upvalue_count {
                        let is_local = self.read_byte() != 0;
                        let index = self.read_byte() as usize;
                        if is_local {
                            let upvalue = self.capture_upvalue(index)?;
                            closure.borrow_mut().upvalues.push(upvalue);
                        } else {
                            let current_closure_upvalue =
                                current_closure.borrow().upvalues[index].clone();
                            closure.borrow_mut().upvalues.push(current_closure_upvalue);
                        }
                    }
                }
                OpCode::CloseUpvalue => {
                    self.close_upvalues(self.store.value_stack_top - 1)?;
                    self.pop_value();
                }
                OpCode::Return => {
                    let result = self.pop_value();
                    let slots = self.current_frame().slots;
                    self.close_upvalues(slots)?;
                    let start_index = self.pop_frame().ok_or(Error::Runtime)?.start_stack_index;
                    if self.store.frame_stack_top == 0 {
                        return Ok(());
                    }
                    self.store.value_stack_top = start_index;
                    self.push_value(result);
                }
                OpCode::Class => {
                    let ConstantValue::String(name) = &*self.read_constant() else {
                        panic!("IVME: Unexpected constant value.")
                    };
                    let class = self.new_class(name);
                    self.push_value(class);
                }
                OpCode::Inherit => {
                    let Ok(superclass) = self.peek_typed::<Pointer<ObjClass>>(1) else {
                        self.runtime_error("Superclass must be a class.\n".into());
                        return Err(Error::Runtime);
                    };
                    let subclass = self.peek_typed::<Pointer<ObjClass>>(0)?;
                    let mut methods: Vec<_> = superclass
                        .borrow()
                        .methods
                        .iter()
                        .cloned()
                        .collect::<Vec<_>>();
                    methods.retain(|x| {
                        x.clone()
                            .is_some_and(|y| y.key.is_some() && y.value.is_some())
                    });
                    let methods = methods
                        .into_iter()
                        .map(Option::unwrap)
                        .map(|x| (x.key.unwrap(), x.value.unwrap()))
                        .collect::<Vec<_>>();
                    for (key, value) in methods {
                        subclass.borrow_mut().methods.insert(key, value);
                    }
                    self.pop_value(); // Subclass
                }
                OpCode::Method => {
                    let ConstantValue::String(name) = &*self.read_constant() else {
                        panic!("IVME: Unexpected constant value.")
                    };
                    self.define_method(name)?;
                }
                OpCode::Unknown => return Err(Error::Runtime),
            }
        }
    }

    fn call(&mut self, closure: Pointer<ObjClosure>, arg_count: usize) -> Result<(), Error> {
        let closure_ref = closure.borrow();
        let arity = closure_ref.function.borrow().arity;

        if arg_count != arity {
            self.runtime_error(format!(
                "Expected {} arguments but got {}.\n",
                arity, arg_count
            ));
            return Err(Error::Runtime);
        }
        if self.store.frame_stack_top == MAX_FRAMES {
            self.runtime_error("Stack overflow.\n".into());
            return Err(Error::Runtime);
        }
        let frame = &mut self.store.frame_stack[self.store.frame_stack_top];
        *frame = CallFrame {
            closure: Some(closure.clone()),
            ip: 0,
            slots: arg_count,
            start_stack_index: self.store.value_stack_top - 1 - arg_count,
        };
        self.store.frame_stack_top += 1;
        Ok(())
    }

    #[inline]
    fn new_class(&mut self, name: &ObjString) -> Pointer<ObjClass> {
        let name_ref = self.store.insert_string(name.clone());
        let class = ObjClass {
            name: name_ref,
            methods: Table::default(),
        };
        self.store.insert_class(class)
    }

    #[inline]
    fn new_closure(&mut self, function: Pointer<ObjFunction>) -> Pointer<ObjClosure> {
        let function_ref = function.borrow();
        let upvalues = Vec::with_capacity(function_ref.upvalue_count);
        let closure = ObjClosure {
            function: function.clone(),
            upvalues,
        };
        self.store.insert_closure(closure)
    }

    #[inline]
    fn new_instance(&mut self, class: Pointer<ObjClass>) -> Pointer<ObjInstance> {
        let instance = ObjInstance {
            class,
            fields: Table::default(),
        };
        self.store.insert_instance(instance)
    }

    #[inline]
    fn new_bound_method(
        &mut self,
        receiver: RuntimeValue,
        method: Pointer<ObjClosure>,
    ) -> Pointer<ObjBoundMethod> {
        let bound_method = ObjBoundMethod { receiver, method };
        self.store.insert_bound_method(bound_method)
    }

    #[inline]
    fn new_native(&mut self, function: NativeFn) -> Pointer<ObjNative> {
        self.store.insert_native(ObjNative { function })
    }

    #[inline]
    fn push_value(&mut self, value: impl Into<RuntimeValue>) {
        if self.store.value_stack_top == MAX_STACK_SIZE {
            panic!("IVME: Stack overflow.");
        }
        self.store.value_stack[self.store.value_stack_top] = value.into();
        self.store.value_stack_top += 1;
    }

    #[inline]
    fn pop_frame(&mut self) -> Option<CallFrame> {
        if self.store.frame_stack_top == 0 {
            return None;
        }
        self.store.frame_stack_top -= 1;
        Some(self.store.frame_stack[self.store.frame_stack_top].clone())
    }

    #[inline]
    fn pop_value(&mut self) -> RuntimeValue {
        self.store.value_stack_top -= 1;
        self.store.value_stack[self.store.value_stack_top].clone()
    }

    #[inline]
    fn peek_value(&mut self, distance: usize) -> &mut RuntimeValue {
        let index = self.store.value_stack_top - 1 - distance;
        &mut self.store.value_stack[index]
    }

    #[inline]
    fn peek_typed<T: TryFrom<RuntimeValue, Error = Error>>(
        &mut self,
        distance: usize,
    ) -> Result<T, Error> {
        self.peek_value(distance).clone().try_into()
    }

    #[inline]
    fn pop_typed<T: TryFrom<RuntimeValue, Error = Error>>(&mut self) -> T {
        self.pop_value()
            .try_into()
            .expect("IVME: failed to convert value")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, Default)]
    struct TestOut {
        buf: Vec<u8>,
        flushed: Vec<String>,
    }

    impl Write for TestOut {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.buf = buf.into();
            Ok(self.buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            let buf = self.buf.clone();
            self.flushed.push(String::from_utf8(buf).unwrap());
            self.buf = Vec::new();
            Ok(())
        }
    }

    #[test]
    fn it_runs_an_empty_program() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = "";
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect("Failed to run empty program");
        assert!(vm.out.flushed.is_empty());
        assert!(vm.e_out.flushed.is_empty());
    }

    #[test]
    fn it_runs_a_program_with_a_single_expression_statement() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = "1;";
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect("Failed to run program");
        assert!(vm.out.flushed.is_empty());
        assert!(vm.e_out.flushed.is_empty());
    }

    #[test]
    fn it_runs_a_program_with_a_print_statement() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = "print 1;";
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect("Failed to run program");

        assert!(!vm.out.flushed.is_empty());
        assert!(vm.e_out.flushed.is_empty());
        assert_eq!(vm.out.flushed[0], "1\n".to_string());
    }

    #[test]
    fn it_runs_a_program_with_scopes() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = r#"
            var a = 1; 
            { 
                var b = a; 
                print b;
            }
            "#;
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect("Failed to run program");
        assert!(!vm.out.flushed.is_empty());
        assert!(vm.e_out.flushed.is_empty());
        assert_eq!(vm.out.flushed[0], "1\n".to_string());
    }

    #[test]
    fn it_runs_a_program_with_functions() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = r#"
            fun foo(a, b, c) { 
                print a + b + c; 
            } 
            print foo(1, 2, 3);
        "#;
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect("Failed to run program");
        assert!(!vm.out.flushed.is_empty());
        assert!(vm.e_out.flushed.is_empty());
        assert_eq!(vm.out.flushed.len(), 2);
        assert_eq!(vm.out.flushed[0], "6\n".to_string());
        assert_eq!(vm.out.flushed[1], "nil\n".to_string());
    }

    #[test]
    fn it_runs_a_program_with_control_flow() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = r#"
            var a = 1; 
            if (true) { 
                a = 2; 
            } else { 
                a = 3; 
            } 
            print a; 
            if (false) { 
                a = 4; 
            } else { 
                a = 6; 
            } 
            print a;
        "#;
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect("Failed to run program");
        assert!(!vm.out.flushed.is_empty());
        assert!(vm.e_out.flushed.is_empty());
        assert_eq!(vm.out.flushed.len(), 2);
        assert_eq!(vm.out.flushed[0], "2\n".to_string());
        assert_eq!(vm.out.flushed[1], "6\n".to_string());
    }

    #[test]
    fn it_runs_a_program_with_a_loop() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = r#"
            for (var b = 1; b < 4; b = b + 1) {
                print b;
            }
        "#;
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect("Failed to run program");
        assert!(!vm.out.flushed.is_empty());
        assert!(vm.e_out.flushed.is_empty());
        assert_eq!(vm.out.flushed.len(), 3);
        assert_eq!(vm.out.flushed[0], "1\n".to_string());
        assert_eq!(vm.out.flushed[1], "2\n".to_string());
        assert_eq!(vm.out.flushed[2], "3\n".to_string());
    }

    #[test]
    fn it_runs_a_program_with_negation() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = r#"
            print -1;
        "#;
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect("Failed to run program");
        assert!(!vm.out.flushed.is_empty());
        assert!(vm.e_out.flushed.is_empty());
        assert_eq!(vm.out.flushed.len(), 1);
        assert_eq!(vm.out.flushed[0], "-1\n".to_string());
    }

    #[test]
    fn it_runs_a_program_with_simple_binary_ops() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = r#"
            print 1 + 2; 
            print 3 * 4; 
            print 5 / 6; 
            print 7 - 8; 
            print 1 == 2; 
            print 1 == 1; 
            print 1 != 1; 
            print 1 != 2; 
            print 1 < 1; 
            print 1 < 2; 
            print 1 < 0; 
            print 1 <= 2; 
            print 1 <= 1; 
            print 1 <= 0; 
            print 1 > 2; 
            print 1 > 1; 
            print 1 > 0; 
            print 1 >= 2; 
            print 1 >= 1; 
            print 1 >= 0; 
            print true and true; 
            print true and false; 
            print true or true; 
            print true or false; 
            print false or false; 
            print "a" + "b";
        "#;
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect("Failed to run program");
        assert!(!vm.out.flushed.is_empty());
        assert!(vm.e_out.flushed.is_empty());
        assert_eq!(vm.out.flushed[0], "3\n".to_string()); // 1 + 2
        assert_eq!(vm.out.flushed[1], "12\n".to_string()); // 3 * 4
        assert_eq!(vm.out.flushed[2], "0.833333\n".to_string()); // 5 / 6
        assert_eq!(vm.out.flushed[3], "-1\n".to_string()); // 7 - 8
        assert_eq!(vm.out.flushed[4], "false\n".to_string()); // 1 == 2
        assert_eq!(vm.out.flushed[5], "true\n".to_string()); // 1 == 1
        assert_eq!(vm.out.flushed[6], "false\n".to_string()); // 1 != 1
        assert_eq!(vm.out.flushed[7], "true\n".to_string()); // 1 != 2
        assert_eq!(vm.out.flushed[8], "false\n".to_string()); // 1 < 1
        assert_eq!(vm.out.flushed[9], "true\n".to_string()); // 1 < 2
        assert_eq!(vm.out.flushed[10], "false\n".to_string()); // 1 < 0
        assert_eq!(vm.out.flushed[11], "true\n".to_string()); // 1 <= 2
        assert_eq!(vm.out.flushed[12], "true\n".to_string()); // 1 <= 1
        assert_eq!(vm.out.flushed[13], "false\n".to_string()); // 1 <= 0
        assert_eq!(vm.out.flushed[14], "false\n".to_string()); // 1 > 2
        assert_eq!(vm.out.flushed[15], "false\n".to_string()); // 1 > 1
        assert_eq!(vm.out.flushed[16], "true\n".to_string()); // 1 > 0
        assert_eq!(vm.out.flushed[17], "false\n".to_string()); // 1 >= 2
        assert_eq!(vm.out.flushed[18], "true\n".to_string()); // 1 >= 1
        assert_eq!(vm.out.flushed[19], "true\n".to_string()); // 1 >= 0
        assert_eq!(vm.out.flushed[20], "true\n".to_string()); // true and true
        assert_eq!(vm.out.flushed[21], "false\n".to_string()); // true and false
        assert_eq!(vm.out.flushed[22], "true\n".to_string()); // true or true
        assert_eq!(vm.out.flushed[23], "true\n".to_string()); // true or false
        assert_eq!(vm.out.flushed[24], "false\n".to_string()); // false or false
        assert_eq!(vm.out.flushed[25], "ab\n".to_string()); // "a" + "b"
    }

    #[test]
    fn it_runs_a_program_with_a_closure() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = r#"
            fun makeClosure(value) { 
                fun closure() { 
                    print value; 
                } 
                return closure; 
            } 
            var doughnut = makeClosure("doughnut"); 
            var bagel = makeClosure("bagel"); 
            doughnut(); 
            bagel();
        "#;
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect("Failed to run program");
        assert!(!vm.out.flushed.is_empty());
        assert_eq!(vm.out.flushed.len(), 2);
        assert!(vm.e_out.flushed.is_empty());
        assert_eq!(vm.out.flushed[0], "doughnut\n".to_string());
        assert_eq!(vm.out.flushed[1], "bagel\n".to_string());
    }

    #[test]
    fn it_runs_a_program_with_a_closure_with_inner_assignment() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = r#"
            fun makeClosure(value) { 
                fun closure(b) { 
                    value = b; 
                    print value;
                } 
                return closure; 
            } 
            var breakfast = "eggs";
            var doughnut = makeClosure(breakfast); 
            var bagel = makeClosure(breakfast); 
            doughnut("doughnut"); 
            bagel("bagel");
        "#;
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect("Failed to run program");
        assert!(!vm.out.flushed.is_empty());
        assert_eq!(vm.out.flushed.len(), 2);
        assert!(vm.e_out.flushed.is_empty());
        assert_eq!(vm.out.flushed[0], "doughnut\n".to_string());
        assert_eq!(vm.out.flushed[1], "bagel\n".to_string());
    }

    #[test]
    fn it_runs_a_program_with_a_deelpy_nested_closure() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = r#"
            var value;
            fun makeClosure() { 
                fun closure(b) { 
                    fun deepClosure(c) {
                        value = b + c;
                    }
                    return deepClosure;
                } 
                return closure; 
            }
            {
                var deep = makeClosure();
                deep(1)(2);
                print value;
            }
        "#;
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect("Failed to run program");
        assert!(!vm.out.flushed.is_empty());
        assert_eq!(vm.out.flushed.len(), 1);
        assert!(vm.e_out.flushed.is_empty());
        assert_eq!(vm.out.flushed[0], "3\n".to_string());
    }

    #[test]
    fn it_runs_a_program_with_a_class_definition() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = r#"
            class TestClass {} 
            print TestClass;
        "#;
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect("Failed to run program");
        assert!(!vm.out.flushed.is_empty());
        assert_eq!(vm.out.flushed.len(), 1);
        assert!(vm.e_out.flushed.is_empty());
        assert_eq!(vm.out.flushed[0], "TestClass\n".to_string());
    }

    #[test]
    fn it_runs_a_program_with_a_class_instance() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = r#"
            class TestClass {} 
            print TestClass();
        "#;
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect("Failed to run program");
        assert!(!vm.out.flushed.is_empty());
        assert_eq!(vm.out.flushed.len(), 1);
        assert!(vm.e_out.flushed.is_empty());
        assert_eq!(vm.out.flushed[0], "TestClass instance\n".to_string());
    }

    #[test]
    fn it_runs_a_program_with_a_class_initializer() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = r#"
            class TestClass { 
                init() { 
                    this.a = 1; 
                    this.b = "b"; 
                } 
            } 
            var instance = TestClass(); 
            print instance.a; 
            print instance.b;
        "#;
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect("Failed to run program");
        assert!(!vm.out.flushed.is_empty());
        assert_eq!(vm.out.flushed.len(), 2);
        assert!(vm.e_out.flushed.is_empty());
        assert_eq!(vm.out.flushed[0], "1\n".to_string());
        assert_eq!(vm.out.flushed[1], "b\n".to_string());
    }

    #[test]
    fn it_runs_a_program_with_a_class_method() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = r#"
            class TestClass { 
                init(c) { 
                    this.c = c; 
                } 
                m(a, b) { 
                    return a + b + this.c; 
                } 
            } 
            var instance = TestClass(5); 
            print instance.m(1, 2);
        "#;
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect("Failed to run program");
        assert!(!vm.out.flushed.is_empty());
        assert_eq!(vm.out.flushed.len(), 1);
        assert!(vm.e_out.flushed.is_empty());
        assert_eq!(vm.out.flushed[0], "8\n".to_string());
    }

    #[test]
    fn it_runs_a_program_with_a_sub_class_super_method() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = r#"
            class ParentClass { 
                init(a) { 
                    this.a = a; 
                } 
                m() { 
                    print this.a; 
                } 
            } 
            class ChildClass < ParentClass { 
                m() { 
                    super.m(); 
                    print this.a + 1; 
                }
            } 
            var child = ChildClass(1); 
            child.m();
        "#;
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect("Failed to run program");
        assert!(!vm.out.flushed.is_empty());
        assert_eq!(vm.out.flushed.len(), 2);
        assert!(vm.e_out.flushed.is_empty());
        assert_eq!(vm.out.flushed[0], "1\n".to_string());
        assert_eq!(vm.out.flushed[1], "2\n".to_string());
    }

    #[test]
    fn it_runs_a_program_with_a_sub_class_super_property() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = r#"
            class ParentClass { 
                init(a) { 
                    this.a = a; 
                } 
                m() { 
                    print this.a; 
                } 
            } 

            class ChildClass < ParentClass { 
                m() { 
                    super.m(); 
                    print super.m; 
                } 
            } 
            var child = ChildClass(1); 
            child.m();
        "#;
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect("Failed to run program");
        assert!(!vm.out.flushed.is_empty());
        assert_eq!(vm.out.flushed.len(), 2);
        assert!(vm.e_out.flushed.is_empty());
        assert_eq!(vm.out.flushed[0], "1\n".to_string());
        assert_eq!(vm.out.flushed[1], "<fn m>\n".to_string());
    }

    #[test]
    fn it_runs_a_program_with_a_native_function() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = "print clock();";
        let mut vm = VM::new(out, e_out);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64()
            .round();
        vm.interpret(source).expect("Failed to run program");
        assert!(!vm.out.flushed.is_empty());
        assert!(vm.e_out.flushed.is_empty());
        let printed_time = vm.out.flushed[0].trim().parse::<f64>().unwrap().round();
        assert!((printed_time - 1.0..printed_time + 1.0).contains(&now));
    }

    #[test]
    fn it_runs_a_program_with_a_native_function_print() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = "print clock;";
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect("Failed to run program");
        assert!(!vm.out.flushed.is_empty());
        assert_eq!(vm.out.flushed.len(), 1);
        assert!(vm.e_out.flushed.is_empty());
        assert_eq!(vm.out.flushed[0], "<native fn>\n");
    }

    #[test]
    fn it_runs_a_program_with_a_function_print() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = r#"
            fun foo() {}
            print foo;
        "#;
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect("Failed to run program");
        assert!(!vm.out.flushed.is_empty());
        assert_eq!(vm.out.flushed.len(), 1);
        assert!(vm.e_out.flushed.is_empty());
        assert_eq!(vm.out.flushed[0], "<fn foo>\n");
    }

    #[test]
    fn it_reports_a_runtime_error_foo() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = r#"
            class TestClass {} 
            var a = TestClass(); 
            print a.foo();
        "#;
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect_err("Expected runtime error");
        assert!(vm.out.flushed.is_empty());
        assert!(!vm.e_out.flushed.is_empty());
        assert_eq!(vm.e_out.flushed.len(), 3);
        assert_eq!(
            vm.e_out.flushed[0],
            "Undefined property 'foo'.\n".to_string()
        );
        assert_eq!(vm.e_out.flushed[1], "[line 4] in ".to_string());
        assert_eq!(vm.e_out.flushed[2], "script\n".to_string());
    }

    #[test]
    fn it_reports_a_runtime_error_non_instance_field_get() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = r#"
            var a = 1; 
            print a.b;
        "#;
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect_err("Expected runtime error");
        assert!(vm.out.flushed.is_empty());
        assert!(!vm.e_out.flushed.is_empty());
        assert_eq!(vm.e_out.flushed.len(), 3);
        assert_eq!(vm.e_out.flushed[0], "Only instances have fields.\n");
        assert_eq!(vm.e_out.flushed[1], "[line 3] in ".to_string());
        assert_eq!(vm.e_out.flushed[2], "script\n".to_string());
    }

    #[test]
    fn it_reports_a_runtime_error_non_instance_field_set() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = r#"
            var a = 1; 
            a.b = 2;
        "#;
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect_err("Expected runtime error");
        assert!(vm.out.flushed.is_empty());
        assert!(!vm.e_out.flushed.is_empty());
        assert_eq!(vm.e_out.flushed.len(), 3);
        assert_eq!(vm.e_out.flushed[0], "Only instances have fields.\n");
        assert_eq!(vm.e_out.flushed[1], "[line 3] in ".to_string());
        assert_eq!(vm.e_out.flushed[2], "script\n".to_string());
    }

    #[test]
    fn it_reports_a_runtime_error_non_number_add() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = r#"
            var a;
            var b;
            a + b;
        "#;
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect_err("Expected runtime error");
        assert!(vm.out.flushed.is_empty());
        assert!(!vm.e_out.flushed.is_empty());
        assert_eq!(vm.e_out.flushed.len(), 3);
        assert_eq!(
            vm.e_out.flushed[0],
            "Operands must be two numbers or two strings.\n"
        );
        assert_eq!(vm.e_out.flushed[1], "[line 4] in ".to_string());
        assert_eq!(vm.e_out.flushed[2], "script\n".to_string());
    }

    #[test]
    fn it_reports_a_runtime_error_non_number_lt() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = r#"
            "a" < "b";
        "#;
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect_err("Expected runtime error");
        assert!(vm.out.flushed.is_empty());
        assert!(!vm.e_out.flushed.is_empty());
        assert_eq!(vm.e_out.flushed.len(), 3);
        assert_eq!(vm.e_out.flushed[0], "Operands must be numbers.\n");
        assert_eq!(vm.e_out.flushed[1], "[line 2] in ".to_string());
        assert_eq!(vm.e_out.flushed[2], "script\n".to_string());
    }

    #[test]
    fn it_reports_a_runtime_error_non_number_le() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = r#"
            "a" <= "b";
        "#;
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect_err("Expected runtime error");
        assert!(vm.out.flushed.is_empty());
        assert!(!vm.e_out.flushed.is_empty());
        assert_eq!(vm.e_out.flushed.len(), 3);
        assert_eq!(vm.e_out.flushed[0], "Operands must be numbers.\n");
        assert_eq!(vm.e_out.flushed[1], "[line 2] in ".to_string());
        assert_eq!(vm.e_out.flushed[2], "script\n".to_string());
    }

    #[test]
    fn it_reports_a_runtime_error_non_number_gt() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = r#"
            "a" > "b";
        "#;
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect_err("Expected runtime error");
        assert!(vm.out.flushed.is_empty());
        assert!(!vm.e_out.flushed.is_empty());
        assert_eq!(vm.e_out.flushed.len(), 3);
        assert_eq!(vm.e_out.flushed[0], "Operands must be numbers.\n");
        assert_eq!(vm.e_out.flushed[1], "[line 2] in ".to_string());
        assert_eq!(vm.e_out.flushed[2], "script\n".to_string());
    }

    #[test]
    fn it_reports_a_runtime_error_non_number_ge() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = r#"
            "a" >= "b";
        "#;
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect_err("Expected runtime error");
        assert!(vm.out.flushed.is_empty());
        assert!(!vm.e_out.flushed.is_empty());
        assert_eq!(vm.e_out.flushed.len(), 3);
        assert_eq!(vm.e_out.flushed[0], "Operands must be numbers.\n");
        assert_eq!(vm.e_out.flushed[1], "[line 2] in ".to_string());
        assert_eq!(vm.e_out.flushed[2], "script\n".to_string());
    }

    #[test]
    fn it_reports_a_runtime_error_non_number_sub() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = r#"
            "a" - "b";
        "#;
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect_err("Expected runtime error");
        assert!(vm.out.flushed.is_empty());
        assert!(!vm.e_out.flushed.is_empty());
        assert_eq!(vm.e_out.flushed.len(), 3);
        assert_eq!(vm.e_out.flushed[0], "Operands must be numbers.\n");
        assert_eq!(vm.e_out.flushed[1], "[line 2] in ".to_string());
        assert_eq!(vm.e_out.flushed[2], "script\n".to_string());
    }

    #[test]
    fn it_reports_a_runtime_error_non_number_mul() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = r#"
            "a" * "b";
        "#;
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect_err("Expected runtime error");
        assert!(vm.out.flushed.is_empty());
        assert!(!vm.e_out.flushed.is_empty());
        assert_eq!(vm.e_out.flushed.len(), 3);
        assert_eq!(vm.e_out.flushed[0], "Operands must be numbers.\n");
        assert_eq!(vm.e_out.flushed[1], "[line 2] in ".to_string());
        assert_eq!(vm.e_out.flushed[2], "script\n".to_string());
    }

    #[test]
    fn it_reports_a_runtime_error_non_number_div() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = r#"
            "a" / "b";
        "#;
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect_err("Expected runtime error");
        assert!(vm.out.flushed.is_empty());
        assert!(!vm.e_out.flushed.is_empty());
        assert_eq!(vm.e_out.flushed.len(), 3);
        assert_eq!(vm.e_out.flushed[0], "Operands must be numbers.\n");
        assert_eq!(vm.e_out.flushed[1], "[line 2] in ".to_string());
        assert_eq!(vm.e_out.flushed[2], "script\n".to_string());
    }

    #[test]
    fn it_reports_a_runtime_error_non_number_negate() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = r#"
            -"a";
        "#;
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect_err("Expected runtime error");
        assert!(vm.out.flushed.is_empty());
        assert!(!vm.e_out.flushed.is_empty());
        assert_eq!(vm.e_out.flushed.len(), 3);
        assert_eq!(vm.e_out.flushed[0], "Operand must be a number.\n");
        assert_eq!(vm.e_out.flushed[1], "[line 2] in ".to_string());
        assert_eq!(vm.e_out.flushed[2], "script\n".to_string());
    }

    #[test]
    fn it_reports_a_runtime_error_undefined_global_get() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = r#"
            -a;
        "#;
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect_err("Expected runtime error");
        assert!(vm.out.flushed.is_empty());
        assert!(!vm.e_out.flushed.is_empty());
        assert_eq!(vm.e_out.flushed.len(), 3);
        assert_eq!(vm.e_out.flushed[0], "Undefined variable 'a'.\n");
        assert_eq!(vm.e_out.flushed[1], "[line 2] in ".to_string());
        assert_eq!(vm.e_out.flushed[2], "script\n".to_string());
    }

    #[test]
    fn it_reports_a_runtime_error_undefined_global_set() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = r#"
            a = 1;
        "#;
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect_err("Expected runtime error");
        assert!(vm.out.flushed.is_empty());
        assert!(!vm.e_out.flushed.is_empty());
        assert_eq!(vm.e_out.flushed.len(), 3);
        assert_eq!(vm.e_out.flushed[0], "Undefined variable 'a'.\n");
        assert_eq!(vm.e_out.flushed[1], "[line 2] in ".to_string());
        assert_eq!(vm.e_out.flushed[2], "script\n".to_string());
    }

    #[test]
    fn it_reports_a_runtime_error_non_class_super() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = r#"
            var a = 1;
            class A < a {}
        "#;
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect_err("Expected runtime error");
        assert!(vm.out.flushed.is_empty());
        assert!(!vm.e_out.flushed.is_empty());
        assert_eq!(vm.e_out.flushed.len(), 3);
        assert_eq!(vm.e_out.flushed[0], "Superclass must be a class.\n");
        assert_eq!(vm.e_out.flushed[1], "[line 3] in ".to_string());
        assert_eq!(vm.e_out.flushed[2], "script\n".to_string());
    }

    #[test]
    fn it_reports_a_runtime_error_bad_function_arity() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = r#"
            fun foo() {}
            foo(1);
        "#;
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect_err("Expected runtime error");
        assert!(vm.out.flushed.is_empty());
        assert!(!vm.e_out.flushed.is_empty());
        assert_eq!(vm.e_out.flushed.len(), 3);
        assert_eq!(vm.e_out.flushed[0], "Expected 0 arguments but got 1.\n");
        assert_eq!(vm.e_out.flushed[1], "[line 3] in ".to_string());
        assert_eq!(vm.e_out.flushed[2], "script\n".to_string());
    }

    #[test]
    fn it_reports_a_runtime_error_stack_overflow() {
        let out = TestOut::default();
        let e_out = TestOut::default();
        let source = r#"
            fun foo() {foo();}
            foo();
        "#;
        let mut vm = VM::new(out, e_out);
        vm.interpret(source).expect_err("Expected runtime error");
        assert!(vm.out.flushed.is_empty());
        assert!(!vm.e_out.flushed.is_empty());
        assert_eq!(vm.e_out.flushed.len(), 129);
        assert_eq!(vm.e_out.flushed[0], "Stack overflow.\n");
        assert_eq!(vm.e_out.flushed[1], "[line 2] in ".to_string());
        assert_eq!(vm.e_out.flushed[2], "foo\n".to_string());
    }
}
