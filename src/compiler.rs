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

    fn emit_opcode(&mut self, opcode: OpCode) {
        self.emit_byte(opcode as u8);
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

    fn emit_jump(&mut self, opcode: OpCode) -> usize {
        match opcode {
            OpCode::Jump | OpCode::JumpIfFalse => {}
            o => panic!("ICE: Tried to emit jump with non jump condition: {o}"),
        }
        self.emit_opcode(opcode);
        self.emit_byte(0xffu8);
        self.emit_byte(0xffu8);
        self.current_function.chunk.code.len() - 2
    }

    fn emit_loop(&mut self, loop_start: usize) {
        self.emit_opcode(OpCode::Loop);

        let offset = self.current_function.chunk.code.len() - loop_start + 2;
        if offset > u16::MAX as usize {
            self.error("Loop body too large.");
        }

        // High bits
        self.emit_byte(((offset >> 8) & 0xff) as u8);
        // Low bits
        self.emit_byte((offset & 0xff) as u8);
    }

    fn patch_jump(&mut self, offset: usize) {
        // -2 to adjust for the bytecode for the jump itself
        let jump = self.current_function.chunk.code.len() - offset - 2;
        if jump > u16::MAX as usize {
            self.error("Too much code to jump over.");
        }

        // High bits
        self.current_function.chunk.code[offset] = ((jump >> 8) & 0xff) as u8;
        // Low bits
        self.current_function.chunk.code[offset + 1] = (jump & 0xff) as u8;
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

    fn mark_initialized(&mut self) {
        todo!()
    }

    fn identifier_constant(&mut self, name: &Token) -> usize {
        todo!()
    }

    fn parse_variable(&mut self, error_message: &str) -> usize {
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
        let global = self.parse_variable("Expect function name.");
        self.mark_initialized();
        self.function(FunctionType::Function);
        self.define_variable(global as u8);
    }

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.");
        if self.advance_if_eq(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_byte(OpCode::Nil as u8);
        }
        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        );
        self.define_variable(global as u8);
    }

    fn named_variable(&mut self, name: Token) {
        todo!()
    }

    fn statement(&mut self) {
        match self.peek_scanner().kind {
            TokenType::Print => self.print_statement(),
            TokenType::For => self.for_statement(),
            TokenType::If => self.if_statement(),
            TokenType::Return => self.return_statement(),
            TokenType::While => self.while_statement(),
            TokenType::LeftBrace => {
                self.begin_scope();
                self.block();
                self.end_scope();
            }
            _ => self.expression_statement(),
        }
    }

    fn print_statement(&mut self) {
        if !self.advance_if_eq(TokenType::Print) {
            panic!("ICE: Failed to find 'print' token for print statement.");
        }
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        self.emit_byte(OpCode::Print as u8);
    }

    fn for_statement(&mut self) {
        if !self.advance_if_eq(TokenType::For) {
            panic!("ICE: Failed to find 'for' token for 'for' statement.");
        }

        self.begin_scope();
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.");
        match self.peek_scanner().kind {
            TokenType::Semicolon => self.advance_scanner(),
            TokenType::Var => {
                self.advance_scanner();
                self.var_declaration();
            }
            _ => self.expression_statement(),
        }

        let mut loop_start = self.current_function.chunk.code.len();
        let mut exit_jump = -1;
        if !self.advance_if_eq(TokenType::Semicolon) {
            self.expression();
            self.consume(TokenType::Semicolon, "Expect ';' after loop condition.");
            exit_jump = self.emit_jump(OpCode::JumpIfFalse) as isize;
            self.emit_opcode(OpCode::Pop);
        }

        if !self.advance_if_eq(TokenType::RightParen) {
            let body_jump = self.emit_jump(OpCode::Jump);
            let increment_start = self.current_function.chunk.code.len();
            self.expression();
            self.emit_opcode(OpCode::Pop);
            self.consume(TokenType::RightParen, "Expect ')' after for clauses.");
            self.emit_loop(loop_start);
            loop_start = increment_start;
            self.patch_jump(body_jump);
        }

        self.statement();
        self.emit_loop(loop_start);
        if exit_jump != -1 {
            self.patch_jump(exit_jump as usize);
            self.emit_opcode(OpCode::Pop);
        }
        self.end_scope();
    }

    fn if_statement(&mut self) {
        if !self.advance_if_eq(TokenType::If) {
            panic!("ICE: Failed to find 'if' token for if statement.");
        }
        self.consume(TokenType::LeftParen, "Expect '(' after condition.");
        let then_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_opcode(OpCode::Pop);
        self.statement();
        let else_jump = self.emit_jump(OpCode::Jump);
        self.patch_jump(then_jump);
        self.emit_opcode(OpCode::Pop);
        if self.advance_if_eq(TokenType::Else) {
            self.statement();
        }
        self.patch_jump(else_jump);
    }

    fn while_statement(&mut self) {
        if !self.advance_if_eq(TokenType::While) {
            panic!("ICE: Failed to find 'while' token for if statement.");
        }
        let loop_start = self.current_function.chunk.code.len();

        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");

        let exit_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_opcode(OpCode::Pop);
        self.statement();
        self.emit_loop(loop_start);
        self.patch_jump(exit_jump);
        self.emit_opcode(OpCode::Pop);
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after expression.");
        self.emit_opcode(OpCode::Pop);
    }

    fn return_statement(&mut self) {
        if self.current_function_type == FunctionType::Script {
            self.error("Can't return from top-level code.");
        }
        if self.advance_if_eq(TokenType::Semicolon) {
            self.emit_return();
            return;
        }
        if self.current_function_type == FunctionType::Initializer {
            self.error("Can't return a value from an initializer.");
        }
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after return value.");
        self.emit_opcode(OpCode::Return);
    }

    fn method(&mut self) {
        todo!()
    }

    fn function(&mut self, function_type: FunctionType) {
        todo!()
    }

    fn block(&mut self) {
        todo!()
    }

    fn variable(&mut self) {
        todo!()
    }

    fn expression(&mut self) {
        todo!()
    }
}
