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
    pub scope_depth: usize,
    pub locals: [Local; u8::MAX as usize],
    pub local_count: usize,
    pub upvalues: [Upvalue; u8::MAX as usize],
    pub upvalue_count: usize,
}

impl Context {
    pub fn new(function_type: FunctionType, name: Option<String>) -> Self {
        let mut locals = array::from_fn(|_| Local::default());
        let local = &mut locals[0];
        local.depth = 0;
        let mut token = Token::default();
        local.name = if FunctionType::Function != function_type {
            token.lexeme = "this".into();
            token.kind = TokenType::This;
            token
        } else {
            token.lexeme = "".into();
            token.kind = TokenType::Identifier;
            token
        };

        let mut function = ObjFunction::default();
        if function_type != FunctionType::Script {
            function.name = name
        }

        Self {
            function,
            scope_depth: 0,
            function_type,
            local_count: 1,
            locals,
            upvalue_count: 0,
            upvalues: array::from_fn(|_| Upvalue::default()),
        }
    }

    pub fn write(&mut self, byte: u8, line: usize) {
        self.function.chunk.write(byte, line);
    }

    pub fn write_opcode(&mut self, opcode: OpCode, line: usize) {
        self.write(opcode as u8, line);
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
