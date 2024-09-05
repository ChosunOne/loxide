use crate::{
    chunk::OpCode,
    error::Error,
    object::obj_function::ObjFunction,
    scanner::Scanner,
    token::{Token, TokenType},
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

#[derive(Debug)]
pub struct ClassCompiler {
    pub enclosing: Option<Box<ClassCompiler>>,
    pub has_super_class: bool,
}

#[derive(Debug)]
pub struct Compiler<'a> {
    scanner: Peekable<Scanner<'a>>,
    had_error: bool,
    panic_mode: bool,
    current_function: &'a mut ObjFunction<'a>,
    current_function_type: FunctionType,
    current_class_compiler: Option<Box<ClassCompiler>>,
    previous_token: Option<Token>,
    line: usize,
    scope_depth: usize,
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
            current_class_compiler: None,
            line: 1,
            scope_depth: 0,
            had_error: false,
            panic_mode: false,
            previous_token: None,
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

    fn identifiers_equal(a: &Token, b: &Token) -> bool {
        a.lexeme.len() == b.lexeme.len() && a.lexeme == b.lexeme
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

    fn synchronize(&mut self) {
        self.panic_mode = false;

        while self.scanner.peek().unwrap().kind != TokenType::Eof {
            if self
                .previous_token
                .as_ref()
                .is_some_and(|x| x.kind == TokenType::Semicolon)
            {
                return;
            }

            match self.scanner.next() {
                None => break,
                Some(t) => match t.kind {
                    TokenType::Class
                    | TokenType::Fun
                    | TokenType::Var
                    | TokenType::If
                    | TokenType::While
                    | TokenType::Print
                    | TokenType::Return => return,
                    _ => {}
                },
            }
        }
    }

    /// The scanner should never return a `None` value, so we panic if it does
    fn peek_scanner(&mut self) -> &Token {
        self.scanner
            .peek()
            .expect("ICE: Failed to get token from scanner")
    }

    fn advance_scanner(&mut self) {
        self.previous_token = self.scanner.next();
        loop {
            let current_token = self.peek_scanner();
            let lexeme = current_token.lexeme.clone();
            match current_token.kind {
                TokenType::Error => self.error_at_current(&lexeme),
                _ => break,
            }
            self.previous_token = self.scanner.next();
        }
    }

    fn advance_if_eq(&mut self, token_type: TokenType) -> bool {
        if self.peek_scanner().kind == token_type {
            self.advance_scanner();
            return true;
        }
        false
    }

    fn consume(&mut self, token_type: TokenType, message: &str) {
        let next_token = self.peek_scanner();

        if next_token.kind == token_type {
            self.advance_scanner();
            return;
        }

        self.error_at_current(message);
    }

    fn error_at(&mut self, token: &Token, message: &str) {
        if self.panic_mode {
            return;
        }

        self.panic_mode = true;
        eprint!("[line {}] Error", token.line);

        match token.kind {
            TokenType::Eof => eprint!("at end"),
            TokenType::Error => {}
            _ => eprint!(" at {}", token.lexeme),
        }

        eprintln!(": {}", message);

        self.had_error = true;
    }

    fn error(&mut self, message: &str) {
        let at_token = self.previous_token.clone().unwrap();
        self.error_at(&at_token, message);
    }

    fn error_at_current(&mut self, message: &str) {
        let at_token = self.scanner.peek().unwrap().clone();
        self.error_at(&at_token, message);
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;
        todo!()
    }

    fn identifier_constant(&mut self, name: &Token) -> usize {
        todo!()
    }

    fn add_local(&mut self, token: Token) {
        todo!()
    }

    fn declare_variable(&mut self) {
        todo!();
    }

    fn define_variable(&mut self, global: u8) {
        todo!();
    }

    fn declaration(&mut self) {
        match self.peek_scanner().kind {
            TokenType::Class => self.class_declaration(),
            TokenType::Fun => self.fun_declaration(),
            TokenType::Var => self.var_declaration(),
            _ => self.statement(),
        }
        if self.panic_mode {
            self.synchronize();
        }
    }

    fn class_declaration(&mut self) {
        self.advance_scanner();
        self.consume(TokenType::Identifier, "Expect class name.");
        let class_name = self.previous_token.clone().unwrap();
        let name_constant = self.identifier_constant(&class_name);
        self.declare_variable();

        self.emit_bytes(OpCode::Class as u8, name_constant as u8);
        self.define_variable(name_constant as u8);

        let class_compiler = ClassCompiler {
            has_super_class: false,
            enclosing: self.current_class_compiler.take(),
        };
        self.current_class_compiler = Some(Box::new(class_compiler));

        if self.advance_if_eq(TokenType::Less) {
            self.consume(TokenType::Identifier, "Expect superclass name.");
            self.variable();
            if Compiler::identifiers_equal(
                &class_name,
                &self
                    .previous_token
                    .clone()
                    .expect("ICE: Failed to read previous token."),
            ) {
                self.error("A class can't inherit from itself.");
            }

            self.begin_scope();
            self.add_local(Token {
                kind: TokenType::Super,
                lexeme: "super".into(),
                line: self.line,
            });
            self.define_variable(0);

            self.named_variable(class_name.clone());
            self.emit_byte(OpCode::Inherit as u8);
            self.current_class_compiler
                .as_deref_mut()
                .expect("ICE: Failed to get current class compiler.")
                .has_super_class = true;
        }

        self.named_variable(class_name);
        self.consume(TokenType::LeftBrace, "Expect '{' before class body.");

        loop {
            let next_token = self.peek_scanner();
            if next_token.kind == TokenType::RightBrace || next_token.kind == TokenType::Eof {
                break;
            }

            self.method();
        }

        self.consume(TokenType::RightBrace, "Expect '}' after class body.");
        self.emit_byte(OpCode::Pop as u8);
        if self
            .current_class_compiler
            .as_ref()
            .expect("ICE: Failed to get current class compiler.")
            .has_super_class
        {
            self.end_scope();
        }

        let mut class_compiler = self
            .current_class_compiler
            .take()
            .expect("ICE: Failed to get current class compiler.");

        self.current_class_compiler = class_compiler.enclosing.take();
    }

    fn fun_declaration(&mut self) {
        todo!()
    }

    fn var_declaration(&mut self) {
        todo!()
    }

    fn named_variable(&mut self, name: Token) {
        todo!()
    }

    fn statement(&mut self) {
        todo!()
    }

    fn method(&mut self) {
        todo!()
    }

    fn variable(&mut self) {
        todo!()
    }
}
