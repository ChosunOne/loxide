pub mod context;
pub mod local;
pub mod upvalue;

use context::Class;

use crate::{
    chunk::{Chunk, OpCode},
    compiler::context::{Context, FunctionType},
    error::Error,
    object::{obj_function::ObjFunction, obj_string::ObjString, Obj, Object},
    scanner::Scanner,
    token::{Token, TokenType},
    value::Value,
};
use std::iter::Peekable;

#[derive(Debug)]
pub struct Compiler {
    scanner: Peekable<Scanner>,
    had_error: bool,
    panic_mode: bool,
    previous_token: Option<Token>,
    line: usize,
    context_stack: Vec<Context>,
}

impl Compiler {
    pub fn new(source: String) -> Self {
        let scanner = Scanner::new(source).peekable();
        let context_stack = vec![Context::new(FunctionType::Script)];
        Self {
            scanner,
            line: 1,
            had_error: false,
            panic_mode: false,
            previous_token: None,
            context_stack,
        }
    }

    pub fn compile(mut self) -> Result<ObjFunction, Error> {
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

        let context = self.pop_context();
        Ok(context.function)
    }

    fn current_context(&mut self) -> &mut Context {
        self.context_stack
            .last_mut()
            .expect("ICE: Failed to get current context")
    }

    fn current_chunk(&mut self) -> &mut Chunk {
        &mut self.current_context().function.chunk
    }

    fn current_function_type(&mut self) -> FunctionType {
        self.current_context().function_type
    }

    fn current_function(&mut self) -> &mut ObjFunction {
        &mut self.current_context().function
    }

    fn identifiers_equal(a: &Token, b: &Token) -> bool {
        a.lexeme.len() == b.lexeme.len() && a.lexeme == b.lexeme
    }

    fn emit_byte(&mut self, byte: u8) {
        let line = self.line;
        self.current_chunk().write(byte, line);
    }

    fn emit_opcode(&mut self, opcode: OpCode) {
        self.emit_byte(opcode as u8);
    }

    fn emit_bytes(&mut self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn emit_return(&mut self) {
        if self.current_function_type() == FunctionType::Initializer {
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
        self.current_chunk().code.len() - 2
    }

    fn emit_loop(&mut self, loop_start: usize) {
        self.emit_opcode(OpCode::Loop);

        let offset = self.current_chunk().code.len() - loop_start + 2;
        if offset > u16::MAX as usize {
            self.error("Loop body too large.");
        }

        // High bits
        self.emit_byte(((offset >> 8) & 0xff) as u8);
        // Low bits
        self.emit_byte((offset & 0xff) as u8);
    }

    fn emit_constant(&mut self, value: Value) {
        self.emit_opcode(OpCode::Constant);
        let constant = self.make_constant(value);
        self.emit_byte(constant);
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let constant = self.current_chunk().add_constant(value);
        if constant > u8::MAX as usize {
            self.error("Too many constants in one chunk.");
            return 0;
        }
        constant as u8
    }

    fn patch_jump(&mut self, offset: usize) {
        // -2 to adjust for the bytecode for the jump itself
        let jump = self.current_chunk().code.len() - offset - 2;
        if jump > u16::MAX as usize {
            self.error("Too much code to jump over.");
        }

        // High bits
        self.current_chunk().code[offset] = ((jump >> 8) & 0xff) as u8;
        // Low bits
        self.current_chunk().code[offset + 1] = (jump & 0xff) as u8;
    }

    fn pop_context(&mut self) -> Context {
        self.context_stack
            .pop()
            .expect("ICE: Failed to pop context.")
    }

    fn peek_context(&mut self, index: usize) -> &mut Context {
        let index = self.context_stack.len() - index - 1;
        &mut self.context_stack[index]
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
        let context = self.current_context();
        context.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        let line = self.line;
        let context = self.current_context();
        while context.local_count > 0
            && context.locals[context.local_count - 1].depth as usize > context.scope_depth
        {
            if context.locals[context.local_count - 1].is_captured {
                context.write_opcode(OpCode::CloseUpvalue, line);
            } else {
                context.write_opcode(OpCode::Pop, line);
            }
            context.local_count -= 1;
        }
    }

    fn mark_initialized(&mut self) {
        let context = self.current_context();
        if context.scope_depth == 0 {
            return;
        }
        context.locals[context.local_count - 1].depth = context.scope_depth as isize;
    }

    fn resolve_local(&mut self, name: &Token, index: usize) -> Option<usize> {
        let context = self.peek_context(index);
        for i in (0..context.local_count).rev() {
            let local = &context.locals[i];
            if Self::identifiers_equal(name, &local.name) {
                if local.depth == -1 {
                    self.error("can't read local variable in its own initializer.");
                }
                return Some(i);
            }
        }
        None
    }

    fn resolve_upvalue(&mut self, name: &Token, index: usize) -> Option<usize> {
        let local = self.resolve_local(name, index);
        let enclosing_context = self.peek_context(index);
        match local {
            Some(l) => {
                enclosing_context.locals[l].is_captured = true;
                return self.add_upvalue(l, true).into();
            }
            None => {
                if let Some(v) = self.resolve_upvalue(name, index + 1) {
                    return self.add_upvalue(v, false).into();
                }
            }
        }

        None
    }

    fn identifier_constant(&mut self, name: &Token) -> u8 {
        self.make_constant(Value::Object(Box::new(Object::String(ObjString {
            obj: Obj::default(),
            hash: 0,
            chars: name.lexeme.clone(),
        }))))
    }

    fn parse_variable(&mut self, error_message: &str) -> u8 {
        todo!()
    }

    fn add_upvalue(&mut self, index: usize, is_local: bool) -> usize {
        let upvalue_count;
        {
            let context = self.current_context();
            upvalue_count = context.upvalue_count;
            for i in 0..upvalue_count {
                let upvalue = &context.upvalues[i];
                if upvalue.index == index && upvalue.is_local == is_local {
                    return i;
                }
            }
        }

        if upvalue_count == u8::MAX as usize {
            self.error("Too many closure variables in function.");
        }

        let context = self.current_context();

        context.upvalues[upvalue_count].is_local = is_local;
        context.upvalues[upvalue_count].index = index;
        context.function.upvalue_count += 1;
        context.function.upvalue_count
    }

    fn add_local(&mut self, name: Token) {
        let context = self.current_context();
        if context.local_count == u8::MAX as usize {
            self.error("Too many local variables in function.");
            return;
        }

        let local = &mut context.locals[context.local_count];
        context.local_count += 1;
        local.name = name;
        local.depth = -1;
        local.is_captured = false;
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

        self.emit_bytes(OpCode::Class as u8, name_constant);
        self.define_variable(name_constant);

        let class = Class {
            has_super_class: false,
        };
        self.current_context().class_stack.push(class);

        if self.advance_if_eq(TokenType::Less) {
            self.consume(TokenType::Identifier, "Expect superclass name.");
            self.variable(false);
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

            self.named_variable(class_name.clone(), false);
            self.emit_byte(OpCode::Inherit as u8);
            self.current_context().peek_class(0).has_super_class = true;
        }

        self.named_variable(class_name, false);
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
        if self.current_context().peek_class(0).has_super_class {
            self.end_scope();
        }

        self.current_context().pop_class();
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

    fn named_variable(&mut self, name: Token, can_assign: bool) {
        let get_op: OpCode;
        let set_op: OpCode;
        let mut arg = self.resolve_local(&name, 0);
        if arg.is_some() {
            get_op = OpCode::GetLocal;
            set_op = OpCode::SetLocal;
        } else if ({
            arg = self.resolve_upvalue(&name, 0);
            arg
        })
        .is_some()
        {
            get_op = OpCode::GetUpvalue;
            set_op = OpCode::SetUpvalue;
        } else {
            arg = Some(self.identifier_constant(&name) as usize);
            get_op = OpCode::GetGlobal;
            set_op = OpCode::SetGlobal;
        }

        if can_assign && self.advance_if_eq(TokenType::Equal) {
            self.expression();
            self.emit_opcode(set_op);
            self.emit_byte(arg.unwrap() as u8);
            return;
        }

        self.emit_opcode(get_op);
        self.emit_byte(arg.unwrap() as u8);
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

        let mut loop_start = self.current_chunk().code.len();
        let mut exit_jump = -1;
        if !self.advance_if_eq(TokenType::Semicolon) {
            self.expression();
            self.consume(TokenType::Semicolon, "Expect ';' after loop condition.");
            exit_jump = self.emit_jump(OpCode::JumpIfFalse) as isize;
            self.emit_opcode(OpCode::Pop);
        }

        if !self.advance_if_eq(TokenType::RightParen) {
            let body_jump = self.emit_jump(OpCode::Jump);
            let increment_start = self.current_chunk().code.len();
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
        let loop_start = self.current_chunk().code.len();

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
        if self.current_function_type() == FunctionType::Script {
            self.error("Can't return from top-level code.");
        }
        if self.advance_if_eq(TokenType::Semicolon) {
            self.emit_return();
            return;
        }
        if self.current_function_type() == FunctionType::Initializer {
            self.error("Can't return a value from an initializer.");
        }
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after return value.");
        self.emit_opcode(OpCode::Return);
    }

    fn method(&mut self) {
        self.consume(TokenType::Identifier, "Expect method name.");
        let name = self
            .previous_token
            .clone()
            .expect("ICE: Failed to read previous token for method.");
        let constant = self.identifier_constant(&name);
        let function_type = {
            if name.lexeme == "init" {
                FunctionType::Initializer
            } else {
                FunctionType::Method
            }
        };

        self.function(function_type);
        self.emit_opcode(OpCode::Method);
        self.emit_byte(constant);
    }

    fn function(&mut self, function_type: FunctionType) {
        let context = Context::new(function_type);
        self.context_stack.push(context);
        self.begin_scope();

        self.consume(TokenType::LeftParen, "Expect '(' after function name.");
        if self.peek_scanner().kind != TokenType::RightParen {
            loop {
                self.current_function().arity += 1;
                if self.current_function().arity > 255 {
                    self.error_at_current("Can't have more than 255 parameters.");
                }
                let constant = self.parse_variable("Expect parameter name.");
                self.define_variable(constant);
                if !self.advance_if_eq(TokenType::Comma) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expect ')' after parameters.");
        self.consume(TokenType::LeftBrace, "Expect '{' before function body.");

        self.block();
        self.emit_return();
        let context = self.pop_context();
        let constant = self.make_constant(Value::new_function(context.function));
        self.emit_opcode(OpCode::Closure);
        self.emit_byte(constant);
    }

    fn block(&mut self) {
        while self.peek_scanner().kind != TokenType::RightBrace
            && self.peek_scanner().kind != TokenType::Eof
        {
            self.declaration();
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block.");
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(
            self.previous_token
                .clone()
                .expect("ICE: Failed to read previous token."),
            can_assign,
        );
    }

    fn expression(&mut self) {
        todo!()
    }
}
