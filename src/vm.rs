use std::{
    array,
    collections::{BTreeMap, HashMap},
};

use crate::{
    call_frame::CallFrame,
    chunk::OpCode,
    compiler::Compiler,
    error::Error,
    object::{
        ObjBoundMethod, ObjClass, ObjClosure, ObjFunction, ObjInstance, ObjString, ObjUpvalue,
        Pointer, Store,
    },
    value::{ConstantValue, RuntimeValue},
};

const MAX_FRAMES: usize = 64;
const MAX_STACK_SIZE: usize = u8::MAX as usize * MAX_FRAMES;

#[derive(Debug)]
pub struct VM {
    store: Store,
    value_stack: [RuntimeValue; MAX_STACK_SIZE],
    frame_stack: [CallFrame; MAX_FRAMES],
    value_stack_top: usize,
    frame_stack_top: usize,
    globals: HashMap<String, RuntimeValue>,
    open_upvalues: BTreeMap<usize, Pointer<ObjUpvalue>>,
}

impl Default for VM {
    fn default() -> Self {
        Self {
            store: Store::default(),
            value_stack: array::from_fn(|_| RuntimeValue::default()),
            frame_stack: array::from_fn(|_| CallFrame::default()),
            frame_stack_top: 0,
            value_stack_top: 0,
            globals: HashMap::default(),
            open_upvalues: BTreeMap::default(),
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

        let function = Box::pin(function);
        let function_ref = self.store.insert_function_pinned(function);
        self.push_value(function_ref);
        let closure = self.new_closure(function_ref);
        self.pop_value()?;
        self.push_value(closure);
        self.call(closure, 0)?;
        self.run()
    }

    fn reset_stack(&mut self) {
        self.value_stack_top = 0;
        self.frame_stack_top = 0;
        self.open_upvalues = BTreeMap::default();
    }

    fn runtime_error(&mut self, message: String) {
        eprintln!("{message}");

        while let Some(frame) = self.pop_frame() {
            let function = (frame.closure.expect("Failed to get frame closure")).function;
            let line = function.chunk.lines[frame.ip];
            eprint!("[line {line}] in ");
            if let Some(name) = &function.name {
                eprintln!("{name}");
            } else {
                eprintln!("script");
            }
        }

        self.reset_stack();
    }

    fn current_frame(&mut self) -> &mut CallFrame {
        &mut self.frame_stack[self.frame_stack_top]
    }

    fn read_byte(&mut self) -> Result<u8, Error> {
        let closure = self
            .current_frame()
            .closure
            .expect("Failed to get currently executing closure");
        let ip = self.current_frame().ip;
        let code = closure.function.chunk.code[ip];
        self.current_frame().ip += 1;
        Ok(code)
    }

    fn read_short(&mut self) -> Result<u16, Error> {
        let byte_1 = self.read_byte()?;
        let byte_2 = self.read_byte()?;
        Ok((byte_1 as u16) << 8 | (byte_2 as u16))
    }

    fn read_constant(&mut self) -> Result<ConstantValue, Error> {
        let closure = self
            .current_frame()
            .closure
            .expect("Failed to get currently executing closure");
        let index = self.read_byte()?;
        Ok(closure.function.chunk.constants[index as usize].clone())
    }

    fn read_string(&mut self) -> Result<ObjString, Error> {
        let string = match self.read_constant()? {
            ConstantValue::String(s) => ObjString::from(s),
            _ => return Err(Error::Runtime),
        };
        Ok(string)
    }

    fn bind_method(&mut self, class: Pointer<ObjClass>, name: ObjString) -> Result<(), Error> {
        let methods = &class.methods;
        let Some(&method) = methods.get(&name.chars) else {
            self.runtime_error(format!("Undefined property '{}'", &name.chars));
            return Err(Error::Runtime);
        };

        let receiver = *self.peek_value(0)?;
        let bound = self.new_bound_method(receiver, method);
        self.pop_value()?;
        self.push_value(bound);
        Ok(())
    }

    fn capture_upvalue(&mut self, stack_index: usize) -> Result<Pointer<ObjUpvalue>, Error> {
        let absolute_stack_index = self.value_stack_top - stack_index;
        if let Some(&upvalue) = self.open_upvalues.get(&absolute_stack_index) {
            return Ok(upvalue);
        }

        let upvalue = ObjUpvalue::Open {
            location: absolute_stack_index,
        };
        let upvalue_ptr = self.store.insert_upvalue(upvalue);

        self.open_upvalues.insert(absolute_stack_index, upvalue_ptr);
        Ok(upvalue_ptr)
    }

    fn close_upvalues(&mut self, last_stack_index: usize) -> Result<(), Error> {
        // TODO: Iterate over the values in the open upvalues map in sorted order, and stop
        // once the last stack index is reached
        todo!()
    }

    fn define_method(&mut self, name: String) -> Result<(), Error> {
        todo!()
    }

    fn concatenate(&mut self) -> Result<(), Error> {
        todo!()
    }

    fn invoke(&mut self, method_name: String, arg_count: usize) -> Result<(), Error> {
        let receiver = self.peek_typed::<Pointer<ObjInstance>>(arg_count)?;
        let instance_fields = &receiver.fields;
        if let Some(&value) = instance_fields.get(&method_name) {
            self.value_stack[self.value_stack_top - arg_count - 1] = value;
            return self.call_value(value, arg_count);
        }
        let class = receiver.class;
        self.invoke_from_class(class, method_name, arg_count)
    }

    fn invoke_from_class(
        &mut self,
        class: Pointer<ObjClass>,
        method_name: String,
        arg_count: usize,
    ) -> Result<(), Error> {
        let Some(&method) = class.methods.get(&method_name) else {
            self.runtime_error("Undefined property {method_name}.".into());
            return Err(Error::Runtime);
        };
        self.call(method, arg_count)
    }

    fn call_value(&mut self, callee: RuntimeValue, arg_count: usize) -> Result<(), Error> {
        match callee {
            RuntimeValue::BoundMethod(bm) => {
                *self.peek_value(arg_count)? = bm.receiver;
                let method = bm.method;
                self.call(method, arg_count)
            }
            RuntimeValue::Class(class) => {
                let instance = self.new_instance(class);
                *self.peek_value(arg_count)? = instance.into();
                if let Some(&initializer) = class.methods.get("init") {
                    self.call(initializer, arg_count)?;
                } else if arg_count != 0 {
                    self.runtime_error("Expected 0 arguments but got {arg_count}.".into());
                    return Err(Error::Runtime);
                }
                Ok(())
            }
            RuntimeValue::Closure(closure) => self.call(closure, arg_count),
            RuntimeValue::Native(native) => {
                let args = (self.value_stack
                    [self.value_stack_top - arg_count..self.value_stack_top])
                    .to_vec();
                let result = (native.function)(args);
                self.value_stack_top -= arg_count + 1;
                self.push_value(result);
                Ok(())
            }

            _ => {
                self.runtime_error("Can only call functions and classes.".into());
                Err(Error::Runtime)
            }
        }
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
                            self.store.insert_string(obj_string).into()
                        }
                        ConstantValue::Function(f) => {
                            let obj_function = *f;
                            self.store.insert_function(obj_function).into()
                        }
                    };

                    self.push_value(runtime_value);
                }
                OpCode::Nil => self.push_value(RuntimeValue::Nil),
                OpCode::True => self.push_value(RuntimeValue::Bool(true)),
                OpCode::False => self.push_value(RuntimeValue::Bool(false)),
                OpCode::Pop => {
                    self.pop_value()?;
                }
                OpCode::GetLocal => {
                    let slot = self.current_frame().slots - self.read_byte()? as usize;
                    let value = *self.peek_value(slot)?;
                    self.push_value(value);
                }
                OpCode::SetLocal => {
                    let slot = self.current_frame().slots - self.read_byte()? as usize;
                    let value = *self.peek_value(0)?;
                    *self.peek_value(slot)? = value;
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
                    let value = *self.peek_value(0)?;
                    self.globals.insert(name.chars, value);
                }
                OpCode::DefineGlobal => {
                    let name = self.read_string()?;
                    let value = *self.peek_value(0)?;
                    self.globals.insert(name.chars, value);
                }
                OpCode::GetUpvalue => {
                    let slot = self.read_byte()? as usize;
                    let location = {
                        let closure = self
                            .current_frame()
                            .closure
                            .expect("Failed to get currently executing closure");
                        let upvalue = closure.upvalues[slot];
                        match *upvalue {
                            ObjUpvalue::Open { location } => location,
                            ObjUpvalue::Closed { value: _ } => return Err(Error::Runtime),
                        }
                    };
                    self.push_value(location);
                }
                OpCode::SetUpvalue => {
                    let slot = self.read_byte()? as usize;
                    let value = self.peek_typed::<usize>(0)?;
                    let mut closure = self
                        .current_frame()
                        .closure
                        .expect("Failed to get currently executing closure");
                    *closure.upvalues[slot] = ObjUpvalue::Open { location: value };
                }
                OpCode::GetProperty => {
                    let name = self.read_string()?;
                    let instance = {
                        let Ok(instance_ref) = self.peek_typed::<Pointer<ObjInstance>>(0) else {
                            self.runtime_error("Only instances have fields.".into());
                            return Err(Error::Runtime);
                        };
                        instance_ref
                    };
                    if let Some(&v) = instance.fields.get(&name.chars) {
                        self.pop_value()?; // Instance
                        self.push_value(v);
                        continue;
                    }

                    let class = instance.class;
                    self.bind_method(class, name)?;
                }
                OpCode::SetProperty => {
                    let Ok(mut instance) = self.peek_typed::<Pointer<ObjInstance>>(1) else {
                        self.runtime_error("Only instances have fields.".into());
                        return Err(Error::Runtime);
                    };
                    let name = self.read_string()?;
                    let value = *self.peek_value(0)?;
                    instance.fields.insert(name.chars, value);
                    let value = self.pop_value()?;
                    self.pop_value()?; // Instance
                    self.push_value(value);
                }
                OpCode::GetSuper => {
                    let name = self.read_string()?;
                    let superclass = match self.pop_value()? {
                        RuntimeValue::Class(o) => o,
                        _ => return Err(Error::Runtime),
                    };
                    self.bind_method(superclass, name)?;
                }
                OpCode::Equal => {
                    let a = self.pop_value()?;
                    let b = self.pop_value()?;
                    self.push_value(a == b);
                }
                OpCode::Greater => {
                    if self.peek_typed::<f64>(0).is_err() || self.peek_typed::<f64>(1).is_err() {
                        self.runtime_error("Operands must be numbers".into());
                        return Err(Error::Runtime);
                    }
                    let b = self.pop_typed::<f64>()?;
                    let a = self.pop_typed::<f64>()?;
                    self.push_value(a > b);
                }
                OpCode::Less => {
                    if self.peek_typed::<f64>(0).is_err() || self.peek_typed::<f64>(1).is_err() {
                        self.runtime_error("Operands must be numbers".into());
                        return Err(Error::Runtime);
                    }
                    let b = self.pop_typed::<f64>()?;
                    let a = self.pop_typed::<f64>()?;
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
                        self.runtime_error("Operands must be two numbers or two strings.".into());
                        return Err(Error::Runtime);
                    }
                    let b = self.pop_typed::<f64>()?;
                    let a = self.pop_typed::<f64>()?;
                    self.push_value(a + b);
                }
                OpCode::Subtract => {
                    if self.peek_typed::<f64>(0).is_err() || self.peek_typed::<f64>(1).is_err() {
                        self.runtime_error("Operands must be numbers".into());
                        return Err(Error::Runtime);
                    }
                    let b = self.pop_typed::<f64>()?;
                    let a = self.pop_typed::<f64>()?;
                    self.push_value(a - b);
                }
                OpCode::Multiply => {
                    if self.peek_typed::<f64>(0).is_err() || self.peek_typed::<f64>(1).is_err() {
                        self.runtime_error("Operands must be numbers".into());
                        return Err(Error::Runtime);
                    }
                    let b = self.pop_typed::<f64>()?;
                    let a = self.pop_typed::<f64>()?;
                    self.push_value(a * b);
                }
                OpCode::Divide => {
                    if self.peek_typed::<f64>(0).is_err() || self.peek_typed::<f64>(1).is_err() {
                        self.runtime_error("Operands must be numbers".into());
                        return Err(Error::Runtime);
                    }
                    let b = self.pop_typed::<f64>()?;
                    let a = self.pop_typed::<f64>()?;
                    self.push_value(a / b);
                }
                OpCode::Not => {
                    let value = self.pop_value()?;
                    self.push_value(value.is_falsey());
                }
                OpCode::Negate => {
                    let Ok(value) = self.pop_typed::<f64>() else {
                        self.runtime_error("Operand must be a number.".into());
                        return Err(Error::Runtime);
                    };
                    self.push_value(-value);
                }
                OpCode::Print => {
                    let value = self.pop_value()?;
                    match value {
                        RuntimeValue::Bool(b) => println!("{b}"),
                        RuntimeValue::Number(n) => println!("{n}"),
                        RuntimeValue::BoundMethod(bm) => {
                            let method = bm.method.function;
                            println!("{}", *method);
                        }
                        RuntimeValue::Class(class) => {
                            println!("{}", *class.name);
                        }
                        RuntimeValue::Closure(closure) => {
                            println!("{}", *closure);
                        }
                        RuntimeValue::Function(function) => {
                            println!("{}", *function);
                        }
                        RuntimeValue::Instance(instance) => {
                            println!("{} instance", *instance.class.name);
                        }
                        RuntimeValue::Native(native) => {
                            println!("{}", *native);
                        }
                        RuntimeValue::String(string) => {
                            println!("{}", *string);
                        }
                        RuntimeValue::Nil => println!("nil"),
                    }
                }
                OpCode::Jump => {
                    let offset = self.read_short()? as usize;
                    self.current_frame().ip += offset;
                }
                OpCode::JumpIfFalse => {
                    let offset = self.read_short()? as usize;
                    if self.peek_value(0)?.is_falsey() {
                        self.current_frame().ip += offset;
                    }
                }
                OpCode::Loop => {
                    let offset = self.read_short()? as usize;
                    self.current_frame().ip -= offset;
                }
                OpCode::Call => {
                    let arg_count = self.read_byte()? as usize;
                    let callee = *self.peek_value(arg_count)?;
                    self.call_value(callee, arg_count)?;
                    self.pop_frame().ok_or(Error::Runtime)?;
                    return Ok(());
                }
                OpCode::Invoke => {
                    let method_name = self.read_string()?;
                    let arg_count = self.read_byte()? as usize;
                    self.invoke(method_name.chars, arg_count)?;
                    self.pop_frame().ok_or(Error::Runtime)?;
                    return Ok(());
                }
                OpCode::SuperInvoke => {
                    let method_name = self.read_string()?;
                    let arg_count = self.read_byte()? as usize;
                    let class = self.pop_typed::<Pointer<ObjClass>>()?;
                    self.invoke_from_class(class, method_name.chars, arg_count)?;
                    self.pop_frame().ok_or(Error::Runtime)?;
                }
                OpCode::Closure => {
                    let ConstantValue::Function(function) = self.read_constant()? else {
                        return Err(Error::Runtime);
                    };
                    let function = self.store.insert_function(*function);
                    let mut closure = self.new_closure(function);
                    self.push_value(closure);
                    let upvalue_count = closure.upvalues.len();
                    let current_closure = self
                        .current_frame()
                        .closure
                        .expect("Failed to get currently executing closure");
                    for i in 0..upvalue_count {
                        let is_local = self.read_byte()? != 0;
                        let index = self.read_byte()? as usize;
                        if is_local {
                            let stack_index = self.current_frame().slots - index;
                            let upvalue = self.capture_upvalue(stack_index)?;
                            closure.upvalues[i] = upvalue;
                        } else {
                            let current_closure_upvalue = current_closure.upvalues[index];
                            closure.upvalues[i] = current_closure_upvalue;
                        }
                    }
                }
                OpCode::CloseUpvalue => {
                    let value = self.peek_typed::<usize>(0)?;
                    self.close_upvalues(value)?;
                    self.pop_value()?;
                }
                OpCode::Return => {
                    let result = self.pop_value()?;
                    let slots = self.current_frame().slots;
                    self.close_upvalues(slots)?;
                    self.pop_frame().ok_or(Error::Runtime)?;
                    if self.frame_stack_top == 0 {
                        return Ok(());
                    }
                    self.frame_stack_top -= slots;
                    self.push_value(result);
                }
                OpCode::Class => {
                    let name = self.read_string()?;
                    let class = self.new_class(name);
                    self.push_value(class);
                }
                OpCode::Inherit => {
                    let Ok(superclass) = self.peek_typed::<Pointer<ObjClass>>(1) else {
                        self.runtime_error("Superclass must be a class.".into());
                        return Err(Error::Runtime);
                    };
                    let mut subclass = self.peek_typed::<Pointer<ObjClass>>(0)?;
                    let methods: Vec<_> = superclass
                        .methods
                        .iter()
                        .map(|x| (x.0.clone(), *x.1))
                        .collect();
                    for (key, value) in methods {
                        subclass.methods.insert(key, value);
                    }
                    self.pop_value()?; // Subclass
                }
                OpCode::Method => {
                    let name = self.read_string()?;
                    self.define_method(name.chars)?;
                }
                OpCode::Unknown => return Err(Error::Runtime),
            }
        }
    }

    fn call(&mut self, closure: Pointer<ObjClosure>, arg_count: usize) -> Result<(), Error> {
        let arity = closure.function.arity;
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
        frame.closure = Some(closure);
        frame.slots = arg_count;
        self.frame_stack_top += 1;
        Ok(())
    }

    fn new_class(&mut self, name: ObjString) -> Pointer<ObjClass> {
        let name_ref = self.store.insert_string(name);
        let class = ObjClass {
            name: name_ref,
            methods: HashMap::new(),
        };
        self.store.insert_class(class)
    }

    fn new_closure(&mut self, function: Pointer<ObjFunction>) -> Pointer<ObjClosure> {
        let upvalues = Vec::with_capacity(function.upvalue_count);
        let closure = ObjClosure { function, upvalues };
        self.store.insert_closure(closure)
    }

    fn new_instance(&mut self, class: Pointer<ObjClass>) -> Pointer<ObjInstance> {
        let instance = ObjInstance {
            class,
            fields: HashMap::new(),
        };
        self.store.insert_instance(instance)
    }

    fn new_bound_method(
        &mut self,
        receiver: RuntimeValue,
        method: Pointer<ObjClosure>,
    ) -> Pointer<ObjBoundMethod> {
        let bound_method = ObjBoundMethod { receiver, method };
        self.store.insert_bound_method(bound_method)
    }

    fn push_value(&mut self, value: impl Into<RuntimeValue>) {
        if self.value_stack_top == MAX_STACK_SIZE {
            panic!("Stack overflow.");
        }
        self.value_stack[self.value_stack_top] = value.into();
        self.value_stack_top += 1;
    }

    fn pop_frame(&mut self) -> Option<CallFrame> {
        if self.frame_stack_top == 0 {
            return None;
        }
        self.frame_stack_top -= 1;
        Some(self.frame_stack[self.frame_stack_top])
    }

    fn pop_value(&mut self) -> Result<RuntimeValue, Error> {
        if self.value_stack_top == 0 {
            return Err(Error::Runtime);
        }
        self.value_stack_top -= 1;
        Ok(self.value_stack[self.value_stack_top])
    }

    fn peek_value(&mut self, distance: usize) -> Result<&mut RuntimeValue, Error> {
        if self.value_stack.is_empty() || distance > self.value_stack.len() - 1 {
            return Err(Error::Runtime);
        }
        let index = self.value_stack.len() - 1 - distance;
        self.value_stack.get_mut(index).ok_or(Error::Runtime)
    }

    fn peek_typed<T: TryFrom<RuntimeValue, Error = Error>>(
        &mut self,
        distance: usize,
    ) -> Result<T, Error> {
        (*self.peek_value(distance)?).try_into()
    }

    fn pop_typed<T: TryFrom<RuntimeValue, Error = Error>>(&mut self) -> Result<T, Error> {
        self.pop_value()?.try_into()
    }
}
