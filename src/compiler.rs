use crate::{
    chunk::OpCode, error::Error, object::obj_function::ObjFunction, scanner::Scanner,
    token::TokenType,
};
use std::iter::Peekable;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub enum FunctionType {
    Function,
    Initializer,
    Method,
    #[default]
    Script,
}

pub struct Compiler<'a> {
    scanner: Peekable<Scanner<'a>>,
    had_error: bool,
    panic_mode: bool,
    current_function: &'a mut ObjFunction<'a>,
    current_function_type: FunctionType,
    line: usize,
}

impl<'a> Compiler<'a> {
    pub fn new(
        source: &'a str,
        function: &'a mut ObjFunction<'a>,
        function_type: FunctionType,
    ) -> Self {
        let scanner = Scanner::new(source).peekable();
        Self {
            scanner,
            current_function: function,
            current_function_type: function_type,
            line: 1,
            had_error: false,
            panic_mode: false,
        }
    }

    pub fn compile(mut self) -> Result<&'a mut ObjFunction<'a>, Error> {
        loop {
            match self.scanner.peek() {
                None => break,
                Some(t) => {
                    if t.kind == TokenType::Eof {
                        break;
                    }
                }
            }

            self.declaration();
        }
        if self.had_error {
            return Err(Error::Compile);
        }

        Ok(self.end())
    }

    fn emit_byte(&mut self, byte: u8) {
        self.current_function.chunk.write(byte, self.line);
    }

    fn emit_bytes(&mut self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn emit_return(&mut self) {
        if self.current_function_type == FunctionType::Initializer {
            self.emit_bytes(OpCode::GetLocal as u8, 0);
        } else {
            self.emit_byte(OpCode::Nil as u8);
        }
        self.emit_byte(OpCode::Return as u8);
    }

    fn end(mut self) -> &'a mut ObjFunction<'a> {
        self.emit_return();
        self.current_function
    }

    fn declaration(&mut self) {
        todo!()
    }
}
