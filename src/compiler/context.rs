use std::array;

use crate::{
    chunk::OpCode,
    compiler::{local::Local, upvalue::Upvalue},
    object::ObjFunction,
    token::{Token, TokenType},
};

#[derive(Debug)]
pub struct Context {
    pub function: ObjFunction,
    pub function_type: FunctionType,
    pub class_stack: Vec<Class>,
    pub scope_depth: usize,
    pub locals: [Local; u8::MAX as usize],
    pub local_count: usize,
    pub upvalues: [Upvalue; u8::MAX as usize],
    pub upvalue_count: usize,
}

impl Context {
    pub fn new(function_type: FunctionType) -> Self {
        let mut locals = array::from_fn(|_| Local::default());
        let local = &mut locals[0];
        local.depth = 0;
        let mut token = Token::default();
        local.name = if FunctionType::Function == function_type {
            token.lexeme = "this".into();
            token.kind = TokenType::Identifier;
            token
        } else {
            token.lexeme = "".into();
            token.kind = TokenType::Identifier;
            token
        };

        Self {
            function: ObjFunction::default(),
            scope_depth: 0,
            function_type,
            local_count: 1,
            locals,
            upvalue_count: 0,
            upvalues: array::from_fn(|_| Upvalue::default()),
            class_stack: Vec::new(),
        }
    }

    pub fn write(&mut self, byte: u8, line: usize) {
        self.function.chunk.write(byte, line);
    }

    pub fn write_opcode(&mut self, opcode: OpCode, line: usize) {
        self.write(opcode as u8, line);
    }

    pub fn current_class(&mut self) -> &mut Class {
        self.class_stack
            .last_mut()
            .expect("ICE: Failed to get current class")
    }

    pub fn pop_class(&mut self) -> Class {
        self.class_stack
            .pop()
            .expect("ICE: Failed to get current class")
    }

    pub fn peek_class(&mut self, index: usize) -> &mut Class {
        let index = self.class_stack.len() - index - 1;
        &mut self.class_stack[index]
    }
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub enum FunctionType {
    Function,
    Initializer,
    Method,
    #[default]
    Script,
}

#[derive(Debug)]
pub struct Class {
    pub has_super_class: bool,
}
