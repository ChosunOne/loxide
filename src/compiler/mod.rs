pub mod binding_power;
pub mod context;
pub mod local;
pub mod upvalue;

use binding_power::{BindingPower, InfixBindingPower, PostfixBindingPower, PrefixBindingPower};

use crate::{
    chunk::{Chunk, OpCode},
    compiler::context::{Context, FunctionType},
    error::Error,
    object::obj_function::ObjFunction,
    scanner::Scanner,
    token::{Token, TokenType},
    value::Value,
};
use std::iter::Peekable;

#[derive(Debug)]
pub struct Class {
    pub has_super_class: bool,
}

#[derive(Debug)]
pub struct Compiler {
    scanner: Peekable<Scanner>,
    had_error: bool,
    panic_mode: bool,
    previous_token: Option<Token>,
    line: usize,
    context_stack: Vec<Context>,
    class_stack: Vec<Class>,
}

impl Compiler {
    pub fn new(source: String) -> Self {
        let scanner = Scanner::new(source).peekable();
        let context_stack = vec![Context::new(FunctionType::Script, None)];
        Self {
            scanner,
            line: 1,
            had_error: false,
            panic_mode: false,
            previous_token: None,
            context_stack,
            class_stack: Vec::new(),
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
        self.emit_return();
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

        let offset = (offset as u16).to_le_bytes();

        // High bits
        self.emit_byte(offset[1]);
        // Low bits
        self.emit_byte(offset[0]);
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

    fn peek_context(&mut self, index: usize) -> Option<&mut Context> {
        if index >= self.context_stack.len() {
            return None;
        }
        let index = self.context_stack.len() - index - 1;
        self.context_stack.get_mut(index)
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

    fn previous(&self) -> &Token {
        self.previous_token
            .as_ref()
            .expect("ICE: Failed to read previous token")
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
            TokenType::Eof => eprint!(" at end"),
            TokenType::Error => {}
            _ => eprint!(" at {}", token.lexeme),
        }

        eprintln!(": {}", message);

        self.had_error = true;
    }

    fn error(&mut self, message: &str) {
        let at_token = self.previous().clone();
        self.error_at(&at_token, message);
    }

    fn error_at_current(&mut self, message: &str) {
        let at_token = self.peek_scanner().clone();
        self.error_at(&at_token, message);
    }

    fn begin_scope(&mut self) {
        let context = self.current_context();
        context.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        let line = self.line;
        let context = self.current_context();
        context.scope_depth -= 1;
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
        let context = self.peek_context(index)?;
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
        self.peek_context(index + 1)?;
        let local = self.resolve_local(name, index + 1);
        let enclosing_context = self.peek_context(index + 1)?;
        match local {
            Some(l) => {
                enclosing_context.locals[l].is_captured = true;
                return self.add_upvalue(index, l, true).into();
            }
            None => {
                if let Some(v) = self.resolve_upvalue(name, index + 1) {
                    return self.add_upvalue(index, v, false).into();
                }
            }
        }

        None
    }

    fn identifier_constant(&mut self, name: Token) -> u8 {
        self.make_constant(Value::from(name.lexeme))
    }

    fn parse_variable(&mut self, error_message: &str) -> u8 {
        self.consume(TokenType::Identifier, error_message);
        self.declare_variable();
        if self.current_context().scope_depth > 0 {
            return 0;
        }
        self.identifier_constant(self.previous().clone())
    }

    fn add_upvalue(&mut self, context_index: usize, upvalue_index: usize, is_local: bool) -> usize {
        let upvalue_count = self
            .peek_context(context_index)
            .expect("ICE: Failed to peek context")
            .function
            .upvalue_count;
        {
            let context = self
                .peek_context(context_index)
                .expect("ICE: Failed to peek context");
            for i in 0..upvalue_count {
                let upvalue = &context.upvalues[i];
                if upvalue.index == upvalue_index && upvalue.is_local == is_local {
                    return i;
                }
            }
        }

        if upvalue_count == u8::MAX as usize {
            self.error("Too many closure variables in function.");
            return 0;
        }

        let context = self
            .peek_context(context_index)
            .expect("Failed to peek context");

        context.upvalues[upvalue_count].is_local = is_local;
        context.upvalues[upvalue_count].index = upvalue_index;
        context.function.upvalue_count += 1;
        context.function.upvalue_count - 1
    }

    fn add_local(&mut self, name: Token) {
        let context = self.current_context();
        if context.local_count == u8::MAX as usize {
            self.error("Too many local variables in function.");
            return;
        }

        println!("local_count: {}", context.local_count);
        let local = &mut context.locals[context.local_count];
        context.local_count += 1;
        local.name = name;
        local.depth = -1;
        local.is_captured = false;
    }

    fn declare_variable(&mut self) {
        if self.current_context().scope_depth == 0 {
            return;
        }
        let local_count = self.current_context().local_count;
        let scope_depth = self.current_context().scope_depth;
        let name = self.previous().clone();

        for i in (0..local_count).rev() {
            let locals = &self
                .context_stack
                .last()
                .expect("ICE: Failed to read context stack.")
                .locals;
            let local = &locals[i];
            if local.depth != -1 && (local.depth as usize) < scope_depth {
                break;
            }

            if Self::identifiers_equal(&name, &local.name) {
                self.error("Robert can't make up his mind about whether to allow redefining an existing variable, so he made this an error in the local scope but not in the global one.");
            }
        }
        self.add_local(name);
    }

    fn define_variable(&mut self, global: u8) {
        if self.current_context().scope_depth > 0 {
            self.mark_initialized();
            return;
        }
        self.emit_opcode(OpCode::DefineGlobal);
        self.emit_byte(global);
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
        let class_name = self.previous().clone();
        let name_constant = self.identifier_constant(class_name.clone());
        self.declare_variable();

        self.emit_bytes(OpCode::Class as u8, name_constant);
        self.define_variable(name_constant);

        let class = Class {
            has_super_class: false,
        };
        self.class_stack.push(class);

        if self.advance_if_eq(TokenType::Less) {
            self.consume(TokenType::Identifier, "Expect superclass name.");
            self.variable(BindingPower::LogicalLeft);
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

            self.named_variable(class_name.clone(), BindingPower::LogicalLeft);
            self.emit_opcode(OpCode::Inherit);
            self.peek_class(0).has_super_class = true;
        }

        self.named_variable(class_name, BindingPower::LogicalLeft);
        self.consume(TokenType::LeftBrace, "Expect '{' before class body.");

        loop {
            let next_token = self.peek_scanner();
            if next_token.kind == TokenType::RightBrace || next_token.kind == TokenType::Eof {
                break;
            }

            self.method();
        }

        self.consume(TokenType::RightBrace, "Expect '}' after class body.");
        self.emit_opcode(OpCode::Pop);
        if self.peek_class(0).has_super_class {
            self.end_scope();
        }

        self.pop_class();
    }

    fn fun_declaration(&mut self) {
        self.advance_scanner();
        let global = self.parse_variable("Expect function name.");
        self.mark_initialized();
        self.function(FunctionType::Function);
        self.define_variable(global);
    }

    fn var_declaration(&mut self) {
        self.advance_scanner();
        let global = self.parse_variable("Expect variable name.");
        if self.advance_if_eq(TokenType::Equal) {
            self.expression(BindingPower::AssignmentRight);
        } else {
            self.emit_opcode(OpCode::Nil);
        }
        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        );
        self.define_variable(global);
    }

    fn named_variable(&mut self, name: Token, min_binding_power: BindingPower) {
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
            arg = Some(self.identifier_constant(name) as usize);
            get_op = OpCode::GetGlobal;
            set_op = OpCode::SetGlobal;
        }

        let can_assign = min_binding_power <= BindingPower::AssignmentLeft;
        if can_assign && self.advance_if_eq(TokenType::Equal) {
            self.expression(BindingPower::AssignmentRight);
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
        self.expression(BindingPower::AssignmentRight);
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
                self.var_declaration();
            }
            _ => self.expression_statement(),
        }

        let mut loop_start = self.current_chunk().code.len();
        let mut exit_jump = -1;
        if !self.advance_if_eq(TokenType::Semicolon) {
            self.expression(BindingPower::AssignmentRight);
            self.consume(TokenType::Semicolon, "Expect ';' after loop condition.");
            exit_jump = self.emit_jump(OpCode::JumpIfFalse) as isize;
            self.emit_opcode(OpCode::Pop);
        }

        if !self.advance_if_eq(TokenType::RightParen) {
            let body_jump = self.emit_jump(OpCode::Jump);
            let increment_start = self.current_chunk().code.len();
            self.expression(BindingPower::AssignmentRight);
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
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.");
        self.expression(BindingPower::Group);
        self.consume(TokenType::RightParen, "Expect ')' after condition.");
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
            panic!("ICE: Failed to find 'while' token for while statement.");
        }
        let loop_start = self.current_chunk().code.len();

        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.");
        self.expression(BindingPower::AssignmentRight);
        self.consume(TokenType::RightParen, "Expect ')' after condition.");

        let exit_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_opcode(OpCode::Pop);
        self.statement();
        self.emit_loop(loop_start);
        self.patch_jump(exit_jump);
        self.emit_opcode(OpCode::Pop);
    }

    fn expression_statement(&mut self) {
        self.expression(BindingPower::AssignmentRight);
        self.consume(TokenType::Semicolon, "Expect ';' after expression.");
        self.emit_opcode(OpCode::Pop);
    }

    fn return_statement(&mut self) {
        if !self.advance_if_eq(TokenType::Return) {
            panic!("ICE: Failed to read 'return' token for return statement.");
        }
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
        self.expression(BindingPower::AssignmentRight);
        self.consume(TokenType::Semicolon, "Expect ';' after return value.");
        self.emit_opcode(OpCode::Return);
    }

    fn method(&mut self) {
        self.consume(TokenType::Identifier, "Expect method name.");
        let name = self.previous().clone();
        let constant = self.identifier_constant(name);
        let function_type = {
            if self.previous().lexeme == "init" {
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
        let name = self.previous().lexeme.clone();
        let context = Context::new(function_type, name.into());
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

        self.block();
        self.emit_return();
        let context = self.pop_context();
        let upvalues = &context.upvalues[..context.function.upvalue_count];
        let constant = self.make_constant(Value::from(context.function));
        self.emit_opcode(OpCode::Closure);
        self.emit_byte(constant);
        for upvalue in upvalues {
            self.emit_byte(upvalue.is_local as u8);
            self.emit_byte(upvalue.index as u8);
        }
    }

    fn block(&mut self) {
        if !self.advance_if_eq(TokenType::LeftBrace) {
            panic!("ICE: Failed to find '{{' token for block statement.");
        }
        while self.peek_scanner().kind != TokenType::RightBrace
            && self.peek_scanner().kind != TokenType::Eof
        {
            self.declaration();
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block.");
    }

    fn variable(&mut self, min_binding_power: BindingPower) {
        let name = self.previous().clone();
        self.named_variable(name, min_binding_power);
    }

    fn expression(&mut self, min_binding_power: BindingPower) {
        self.advance_scanner();

        match self.previous().kind {
            TokenType::Identifier => self.variable(min_binding_power),
            TokenType::True | TokenType::False | TokenType::Nil => self.literal(),
            TokenType::Number => self.number(),
            TokenType::String => self.string(),
            TokenType::Super => self.super_(min_binding_power),
            TokenType::This => self.this(min_binding_power),
            _ => {}
        }

        let prefix_binding_power = PrefixBindingPower::try_from(self.previous().kind).ok();
        if let Some(bp) = prefix_binding_power {
            match self.previous().kind {
                TokenType::LeftParen => self.grouping(bp.binding_power),
                TokenType::Minus | TokenType::Bang => self.unary(bp.binding_power),
                _ => {
                    panic!(
                        "ICE: Got token type {:?} but it doesn't have prefix binding power.",
                        self.previous().kind
                    );
                }
            }
        }

        loop {
            let next_token = self.peek_scanner();
            if next_token.kind == TokenType::Eof {
                break;
            }

            if let Ok(bp) = PostfixBindingPower::try_from(next_token.kind) {
                if bp < min_binding_power {
                    break;
                }
                self.advance_scanner();

                // If we have any postfix operators, we would process them here
                continue;
            }

            if let Ok(bp) = InfixBindingPower::try_from(next_token.kind) {
                if bp < min_binding_power {
                    break;
                }
                self.advance_scanner();
                match &self.previous().kind {
                    TokenType::LeftParen => self.call(),
                    TokenType::Dot => self.dot(),
                    TokenType::Minus
                    | TokenType::Plus
                    | TokenType::Slash
                    | TokenType::Star
                    | TokenType::BangEqual
                    | TokenType::EqualEqual
                    | TokenType::Equal
                    | TokenType::Greater
                    | TokenType::GreaterEqual
                    | TokenType::Less
                    | TokenType::LessEqual => self.binary(bp.right_binding_power),
                    TokenType::And => self.and(bp.right_binding_power),
                    TokenType::Or => self.or(bp.right_binding_power),
                    t => panic!(
                        "ICE: Got token type {:?} but it doesn't have infix binding power.",
                        t
                    ),
                }
                continue;
            }

            break;
        }

        if min_binding_power <= BindingPower::AssignmentLeft && self.advance_if_eq(TokenType::Equal)
        {
            self.error("Invalid assignment target.");
        }
    }

    fn grouping(&mut self, min_binding_power: BindingPower) {
        self.expression(min_binding_power);
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn unary(&mut self, min_binding_power: BindingPower) {
        let operator = self.previous().clone();
        self.expression(min_binding_power);
        match operator.kind {
            TokenType::Bang => self.emit_opcode(OpCode::Not),
            TokenType::Minus => self.emit_opcode(OpCode::Negate),
            _ => {}
        }
    }

    fn literal(&mut self) {
        match self.previous().kind {
            TokenType::False => self.emit_opcode(OpCode::False),
            TokenType::Nil => self.emit_opcode(OpCode::Nil),
            TokenType::True => self.emit_opcode(OpCode::True),
            _ => {}
        }
    }

    fn number(&mut self) {
        let num = self
            .previous()
            .lexeme
            .parse()
            .expect("ICE: Failed to parse number.");
        let value = Value::Number(num);
        self.emit_constant(value);
    }

    fn string(&mut self) {
        let value = Value::from(self.previous().lexeme.clone());
        self.emit_constant(value);
    }

    fn super_(&mut self, min_binding_power: BindingPower) {
        if self.class_stack.is_empty() {
            self.error("Can't use 'super' outside of a class.");
        } else if !self.current_class().has_super_class {
            self.error("Can't use 'super' in a class with no superclass.");
        }

        self.consume(TokenType::Dot, "Expect '.' after 'super'.");
        self.consume(TokenType::Identifier, "Expect superclass method name.");
        let name_token = self.previous().clone();
        let name = self.identifier_constant(name_token);
        self.named_variable(
            Token {
                kind: TokenType::This,
                lexeme: "this".into(),
                line: self.line,
            },
            min_binding_power,
        );
        if self.advance_if_eq(TokenType::LeftParen) {
            let arg_count = self.argument_list();
            self.named_variable(
                Token {
                    kind: TokenType::Super,
                    lexeme: "super".into(),
                    line: self.line,
                },
                min_binding_power,
            );
            self.emit_opcode(OpCode::SuperInvoke);
            self.emit_bytes(name, arg_count);
        } else {
            self.named_variable(
                Token {
                    kind: TokenType::Super,
                    lexeme: "super".into(),
                    line: self.line,
                },
                min_binding_power,
            );
            self.emit_opcode(OpCode::GetSuper);
            self.emit_byte(name);
        }
    }

    fn this(&mut self, min_binding_power: BindingPower) {
        if self.class_stack.is_empty() {
            self.error("Can't use 'this' outside of a class.");
            return;
        }
        self.variable(min_binding_power);
    }

    fn call(&mut self) {
        let arg_count = self.argument_list();
        self.emit_opcode(OpCode::Call);
        self.emit_byte(arg_count);
    }

    fn dot(&mut self) {
        self.consume(TokenType::Identifier, "Expect property name after '.'.");
        let name = self.identifier_constant(self.previous().clone());
        if self.advance_if_eq(TokenType::Equal) {
            self.expression(BindingPower::AssignmentRight);
            self.emit_opcode(OpCode::SetProperty);
            self.emit_byte(name);
        } else if self.advance_if_eq(TokenType::LeftParen) {
            let arg_count = self.argument_list();
            self.emit_opcode(OpCode::Invoke);
            self.emit_bytes(name, arg_count);
        } else {
            self.emit_opcode(OpCode::GetProperty);
            self.emit_byte(name);
        }
    }

    fn binary(&mut self, min_binding_power: BindingPower) {
        let operator = self.previous().kind;
        self.expression(min_binding_power);

        match operator {
            TokenType::BangEqual => {
                self.emit_opcode(OpCode::Equal);
                self.emit_opcode(OpCode::Not);
            }
            TokenType::EqualEqual => {
                self.emit_opcode(OpCode::Equal);
            }
            TokenType::Greater => {
                self.emit_opcode(OpCode::Greater);
            }
            TokenType::GreaterEqual => {
                self.emit_opcode(OpCode::Less);
                self.emit_opcode(OpCode::Not);
            }
            TokenType::Less => {
                self.emit_opcode(OpCode::Less);
            }
            TokenType::LessEqual => {
                self.emit_opcode(OpCode::Greater);
                self.emit_opcode(OpCode::Not);
            }
            TokenType::Plus => {
                self.emit_opcode(OpCode::Add);
            }
            TokenType::Minus => {
                self.emit_opcode(OpCode::Subtract);
            }
            TokenType::Star => {
                self.emit_opcode(OpCode::Multiply);
            }
            TokenType::Slash => {
                self.emit_opcode(OpCode::Divide);
            }
            _ => {}
        }
    }

    fn and(&mut self, min_binding_power: BindingPower) {
        let end_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_opcode(OpCode::Pop);
        self.expression(min_binding_power);
        self.patch_jump(end_jump);
    }

    fn or(&mut self, min_binding_power: BindingPower) {
        let else_jump = self.emit_jump(OpCode::JumpIfFalse);
        let end_jump = self.emit_jump(OpCode::Jump);
        self.patch_jump(else_jump);
        self.emit_opcode(OpCode::Pop);
        self.expression(min_binding_power);
        self.patch_jump(end_jump);
    }

    fn argument_list(&mut self) -> u8 {
        let mut arg_count = 0;
        if self.peek_scanner().kind != TokenType::RightParen {
            loop {
                self.expression(BindingPower::AssignmentRight);
                if arg_count == 255 {
                    self.error("Can't have more than 255 arguments.");
                }
                arg_count += 1;
                if !self.advance_if_eq(TokenType::Comma) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expect ')' after arguments.");
        arg_count
    }
}

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use crate::object::{Obj, ObjString, Object};

    use super::*;

    #[test]
    fn it_compiles_an_empty_file() {
        let source = "".into();
        let compiler = Compiler::new(source);
        let function = compiler.compile().unwrap();
        let chunk = function.chunk;

        let expected_codes = [OpCode::Nil as u8, OpCode::Return as u8];
        let expected_lines = [1; 2];

        for (&code, expected_code) in chunk.code.iter().zip(expected_codes) {
            assert_eq!(code, expected_code);
        }
        for (&line, expected_line) in chunk.lines.iter().zip(expected_lines) {
            assert_eq!(line, expected_line);
        }

        assert_eq!(function.arity, 0);
        assert_eq!(function.upvalue_count, 0);
        assert!(function.name.is_none());
        assert_eq!(chunk.code.len(), 2);
        assert_eq!(chunk.lines.len(), 2);
        assert!(chunk.constants.is_empty());
    }

    #[test]
    fn it_compiles_an_empty_block() {
        let source = "{}".into();
        let compiler = Compiler::new(source);
        let function = compiler.compile().unwrap();
        let chunk = function.chunk;

        let expected_codes = [OpCode::Nil as u8, OpCode::Return as u8];
        let expected_lines = [1; 2];

        for (&code, expected_code) in chunk.code.iter().zip(expected_codes) {
            assert_eq!(code, expected_code);
        }
        for (&line, expected_line) in chunk.lines.iter().zip(expected_lines) {
            assert_eq!(line, expected_line);
        }

        assert_eq!(chunk.code.len(), 2);
        assert_eq!(chunk.lines.len(), 2);
        assert!(chunk.constants.is_empty());
    }

    #[test]
    fn it_compiles_an_empty_function() {
        let source = "fun foo() {}".into();
        let compiler = Compiler::new(source);
        let function = compiler.compile().unwrap();
        let chunk = function.chunk;
        let empty_function_value = &chunk.constants[1];
        let Value::Object(b) = empty_function_value else {
            panic!("Failed to get function from chunk.");
        };
        let Object::Function(f) = &**b else {
            panic!("Failed to get function from chunk.");
        };
        let empty_function_chunk = &f.chunk;
        let expected_function_codes = [OpCode::Nil as u8, OpCode::Return as u8];
        let expected_codes = [
            OpCode::Closure as u8,
            1,
            OpCode::DefineGlobal as u8,
            0,
            OpCode::Nil as u8,
            OpCode::Return as u8,
        ];

        let expected_function_lines = [1; 2];
        let expected_lines = [1; 6];

        let expected_function_constants = [];
        let expected_constants = [
            Value::from("foo"),
            Value::from(ObjFunction {
                obj: Obj::default(),
                arity: 0,
                upvalue_count: 0,
                chunk: Chunk {
                    code: expected_function_codes.into(),
                    lines: expected_function_lines.into(),
                    constants: expected_function_constants.clone().into(),
                },
                name: Some(Rc::new(ObjString {
                    obj: Obj::default(),
                    chars: "foo".into(),
                })),
            }),
        ];

        assert_eq!(chunk.code.len(), expected_codes.len());
        for (&code, expected_code) in chunk.code.iter().zip(expected_codes) {
            assert_eq!(code, expected_code);
        }
        assert_eq!(
            empty_function_chunk.code.len(),
            expected_function_codes.len()
        );
        for (&code, expected_code) in empty_function_chunk
            .code
            .iter()
            .zip(expected_function_codes)
        {
            assert_eq!(code, expected_code);
        }

        assert_eq!(chunk.lines.len(), expected_lines.len());
        for (&line, expected_line) in chunk.lines.iter().zip(expected_lines) {
            assert_eq!(line, expected_line);
        }

        assert_eq!(
            empty_function_chunk.lines.len(),
            expected_function_lines.len()
        );
        for (&line, expected_line) in empty_function_chunk
            .lines
            .iter()
            .zip(expected_function_lines)
        {
            assert_eq!(line, expected_line);
        }

        assert_eq!(chunk.constants.len(), expected_constants.len());
        for (constant, expected_constant) in chunk.constants.iter().zip(expected_constants.iter()) {
            assert_eq!(constant, expected_constant);
        }

        assert_eq!(
            empty_function_chunk.constants.len(),
            expected_function_constants.len()
        );
        for (constant, expected_constant) in empty_function_chunk
            .constants
            .iter()
            .zip(expected_function_constants.iter())
        {
            assert_eq!(constant, expected_constant);
        }
    }

    #[test]
    fn it_compiles_a_single_number_literal() {
        let source = "123.456;".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;
        let expected_codes = [
            OpCode::Constant as u8,
            0,
            OpCode::Pop as u8,
            OpCode::Nil as u8,
            OpCode::Return as u8,
        ];
        let expected_lines = [1; 5];
        let expected_constants = [Value::from(123.456)];

        assert_eq!(chunk.code.len(), 5);
        for (&code, expected_code) in chunk.code.iter().zip(expected_codes) {
            assert_eq!(code, expected_code);
        }

        assert_eq!(chunk.lines.len(), 5);
        for (&line, expected_line) in chunk.lines.iter().zip(expected_lines) {
            assert_eq!(line, expected_line);
        }

        assert_eq!(chunk.constants.len(), 1);
        for (constant, expected_constant) in chunk.constants.into_iter().zip(expected_constants) {
            assert_eq!(constant, expected_constant);
        }
    }

    #[test]
    fn it_compiles_a_single_boolean_false_literal() {
        let source = "false;".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;

        let expected_codes = [
            OpCode::False as u8,
            OpCode::Pop as u8,
            OpCode::Nil as u8,
            OpCode::Return as u8,
        ];

        let expected_lines = [1; 4];

        assert_eq!(chunk.code.len(), 4);
        for (&code, expected_code) in chunk.code.iter().zip(expected_codes) {
            assert_eq!(code, expected_code);
        }

        assert_eq!(chunk.lines.len(), 4);
        for (&line, expected_line) in chunk.lines.iter().zip(expected_lines) {
            assert_eq!(line, expected_line);
        }

        assert_eq!(chunk.constants.len(), 0);
    }

    #[test]
    fn it_compiles_a_single_boolean_true_literal() {
        let source = "true;".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;

        let expected_codes = [
            OpCode::True as u8,
            OpCode::Pop as u8,
            OpCode::Nil as u8,
            OpCode::Return as u8,
        ];

        let expected_lines = [1; 4];

        assert_eq!(chunk.code.len(), 4);
        for (&code, expected_code) in chunk.code.iter().zip(expected_codes) {
            assert_eq!(code, expected_code);
        }

        assert_eq!(chunk.lines.len(), 4);
        for (&line, expected_line) in chunk.lines.iter().zip(expected_lines) {
            assert_eq!(line, expected_line);
        }

        assert_eq!(chunk.constants.len(), 0);
    }

    #[test]
    fn it_compiles_a_single_nil_literal() {
        let source = "nil;".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;

        let expected_codes = [
            OpCode::Nil as u8,
            OpCode::Pop as u8,
            OpCode::Nil as u8,
            OpCode::Return as u8,
        ];

        let expected_lines = [1; 4];

        assert_eq!(chunk.code.len(), 4);
        for (&code, expected_code) in chunk.code.iter().zip(expected_codes) {
            assert_eq!(code, expected_code);
        }

        assert_eq!(chunk.lines.len(), 4);
        for (&line, expected_line) in chunk.lines.iter().zip(expected_lines) {
            assert_eq!(line, expected_line);
        }

        assert_eq!(chunk.constants.len(), 0);
    }

    #[test]
    fn it_compiles_a_single_string_literal() {
        let source = "\"hello lox\";".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;
        let expected_codes = [
            OpCode::Constant as u8,
            0,
            OpCode::Pop as u8,
            OpCode::Nil as u8,
            OpCode::Return as u8,
        ];
        let expected_lines = [1; 5];
        let expected_constants = [Value::from("hello lox")];

        assert_eq!(chunk.code.len(), 5);
        for (&code, expected_code) in chunk.code.iter().zip(expected_codes) {
            assert_eq!(code, expected_code);
        }

        assert_eq!(chunk.lines.len(), 5);
        for (&line, expected_line) in chunk.lines.iter().zip(expected_lines) {
            assert_eq!(line, expected_line);
        }

        assert_eq!(chunk.constants.len(), 1);
        for (constant, expected_constant) in chunk.constants.into_iter().zip(expected_constants) {
            assert_eq!(constant, expected_constant);
        }
    }

    #[test]
    fn it_compiles_a_unary_expression() {
        let source = "-1;!true;".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;
        let expected_codes = [
            OpCode::Constant as u8,
            0,
            OpCode::Negate as u8,
            OpCode::Pop as u8,
            OpCode::True as u8,
            OpCode::Not as u8,
            OpCode::Pop as u8,
            OpCode::Nil as u8,
            OpCode::Return as u8,
        ];
        let expected_lines = [1; 9];
        let expected_constants = [Value::from(1.0)];

        assert_eq!(chunk.code.len(), expected_codes.len());
        for (&code, expected_code) in chunk.code.iter().zip(expected_codes) {
            assert_eq!(code, expected_code);
        }

        assert_eq!(chunk.lines.len(), expected_lines.len());
        for (&line, expected_line) in chunk.lines.iter().zip(expected_lines) {
            assert_eq!(line, expected_line);
        }

        assert_eq!(chunk.constants.len(), 1);
        for (constant, expected_constant) in chunk.constants.into_iter().zip(expected_constants) {
            assert_eq!(constant, expected_constant);
        }
    }

    #[test]
    fn it_compiles_an_add_expression() {
        let source = "1 + 2;".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;

        let expected_codes = [
            OpCode::Constant as u8,
            0,
            OpCode::Constant as u8,
            1,
            OpCode::Add as u8,
            OpCode::Pop as u8,
            OpCode::Nil as u8,
            OpCode::Return as u8,
        ];
        let expected_lines = [1; 8];
        let expected_constants = [Value::from(1.0), Value::from(2.0)];

        assert_eq!(chunk.code.len(), expected_codes.len());
        for (&code, expected_code) in chunk.code.iter().zip(expected_codes) {
            assert_eq!(code, expected_code);
        }

        assert_eq!(chunk.lines.len(), expected_lines.len());
        for (&line, expected_line) in chunk.lines.iter().zip(expected_lines) {
            assert_eq!(line, expected_line);
        }

        assert_eq!(chunk.constants.len(), 2);
        for (constant, expected_constant) in chunk.constants.into_iter().zip(expected_constants) {
            assert_eq!(constant, expected_constant);
        }
    }

    #[test]
    fn it_compiles_a_subtract_expression() {
        let source = "1 - 2;".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;

        let expected_codes = [
            OpCode::Constant as u8,
            0,
            OpCode::Constant as u8,
            1,
            OpCode::Subtract as u8,
            OpCode::Pop as u8,
            OpCode::Nil as u8,
            OpCode::Return as u8,
        ];
        let expected_lines = [1; 8];
        let expected_constants = [Value::from(1.0), Value::from(2.0)];

        assert_eq!(chunk.code.len(), expected_codes.len());
        for (&code, expected_code) in chunk.code.iter().zip(expected_codes) {
            assert_eq!(code, expected_code);
        }

        assert_eq!(chunk.lines.len(), expected_lines.len());
        for (&line, expected_line) in chunk.lines.iter().zip(expected_lines) {
            assert_eq!(line, expected_line);
        }

        assert_eq!(chunk.constants.len(), 2);
        for (constant, expected_constant) in chunk.constants.into_iter().zip(expected_constants) {
            assert_eq!(constant, expected_constant);
        }
    }

    #[test]
    fn it_compiles_a_multiply_expression() {
        let source = "1 * 2;".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;

        let expected_codes = [
            OpCode::Constant as u8,
            0,
            OpCode::Constant as u8,
            1,
            OpCode::Multiply as u8,
            OpCode::Pop as u8,
            OpCode::Nil as u8,
            OpCode::Return as u8,
        ];
        let expected_lines = [1; 8];
        let expected_constants = [Value::from(1.0), Value::from(2.0)];

        assert_eq!(chunk.code.len(), expected_codes.len());
        for (&code, expected_code) in chunk.code.iter().zip(expected_codes) {
            assert_eq!(code, expected_code);
        }

        assert_eq!(chunk.lines.len(), expected_lines.len());
        for (&line, expected_line) in chunk.lines.iter().zip(expected_lines) {
            assert_eq!(line, expected_line);
        }

        assert_eq!(chunk.constants.len(), 2);
        for (constant, expected_constant) in chunk.constants.into_iter().zip(expected_constants) {
            assert_eq!(constant, expected_constant);
        }
    }

    #[test]
    fn it_compiles_a_divide_expression() {
        let source = "1 / 2;".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;

        let expected_codes = [
            OpCode::Constant as u8,
            0,
            OpCode::Constant as u8,
            1,
            OpCode::Divide as u8,
            OpCode::Pop as u8,
            OpCode::Nil as u8,
            OpCode::Return as u8,
        ];
        let expected_lines = [1; 8];
        let expected_constants = [Value::from(1.0), Value::from(2.0)];

        assert_eq!(chunk.code.len(), expected_codes.len());
        for (&code, expected_code) in chunk.code.iter().zip(expected_codes) {
            assert_eq!(code, expected_code);
        }

        assert_eq!(chunk.lines.len(), expected_lines.len());
        for (&line, expected_line) in chunk.lines.iter().zip(expected_lines) {
            assert_eq!(line, expected_line);
        }

        assert_eq!(chunk.constants.len(), 2);
        for (constant, expected_constant) in chunk.constants.into_iter().zip(expected_constants) {
            assert_eq!(constant, expected_constant);
        }
    }

    #[test]
    fn it_compiles_a_less_expression() {
        let source = "1 < 2;".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;

        let expected_codes = [
            OpCode::Constant as u8,
            0,
            OpCode::Constant as u8,
            1,
            OpCode::Less as u8,
            OpCode::Pop as u8,
            OpCode::Nil as u8,
            OpCode::Return as u8,
        ];
        let expected_lines = [1; 8];
        let expected_constants = [Value::from(1.0), Value::from(2.0)];

        assert_eq!(chunk.code.len(), expected_codes.len());
        for (&code, expected_code) in chunk.code.iter().zip(expected_codes) {
            assert_eq!(code, expected_code);
        }

        assert_eq!(chunk.lines.len(), expected_lines.len());
        for (&line, expected_line) in chunk.lines.iter().zip(expected_lines) {
            assert_eq!(line, expected_line);
        }

        assert_eq!(chunk.constants.len(), 2);
        for (constant, expected_constant) in chunk.constants.into_iter().zip(expected_constants) {
            assert_eq!(constant, expected_constant);
        }
    }

    #[test]
    fn it_compiles_a_greater_expression() {
        let source = "1 > 2;".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;

        let expected_codes = [
            OpCode::Constant as u8,
            0,
            OpCode::Constant as u8,
            1,
            OpCode::Greater as u8,
            OpCode::Pop as u8,
            OpCode::Nil as u8,
            OpCode::Return as u8,
        ];
        let expected_lines = [1; 8];
        let expected_constants = [Value::from(1.0), Value::from(2.0)];

        assert_eq!(chunk.code.len(), expected_codes.len());
        for (&code, expected_code) in chunk.code.iter().zip(expected_codes) {
            assert_eq!(code, expected_code);
        }

        assert_eq!(chunk.lines.len(), expected_lines.len());
        for (&line, expected_line) in chunk.lines.iter().zip(expected_lines) {
            assert_eq!(line, expected_line);
        }

        assert_eq!(chunk.constants.len(), 2);
        for (constant, expected_constant) in chunk.constants.into_iter().zip(expected_constants) {
            assert_eq!(constant, expected_constant);
        }
    }

    #[test]
    fn it_compiles_an_equal_expression() {
        let source = "1 == 2;".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;

        let expected_codes = [
            OpCode::Constant as u8,
            0,
            OpCode::Constant as u8,
            1,
            OpCode::Equal as u8,
            OpCode::Pop as u8,
            OpCode::Nil as u8,
            OpCode::Return as u8,
        ];
        let expected_lines = [1; 8];
        let expected_constants = [Value::from(1.0), Value::from(2.0)];

        assert_eq!(chunk.code.len(), expected_codes.len());
        for (&code, expected_code) in chunk.code.iter().zip(expected_codes) {
            assert_eq!(code, expected_code);
        }

        assert_eq!(chunk.lines.len(), expected_lines.len());
        for (&line, expected_line) in chunk.lines.iter().zip(expected_lines) {
            assert_eq!(line, expected_line);
        }

        assert_eq!(chunk.constants.len(), 2);
        for (constant, expected_constant) in chunk.constants.into_iter().zip(expected_constants) {
            assert_eq!(constant, expected_constant);
        }
    }

    #[test]
    fn it_compiles_a_less_equal_expression() {
        let source = "1 <= 2;".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;

        let expected_codes = [
            OpCode::Constant as u8,
            0,
            OpCode::Constant as u8,
            1,
            OpCode::Greater as u8,
            OpCode::Not as u8,
            OpCode::Pop as u8,
            OpCode::Nil as u8,
            OpCode::Return as u8,
        ];
        let expected_lines = [1; 9];
        let expected_constants = [Value::from(1.0), Value::from(2.0)];

        assert_eq!(chunk.code.len(), expected_codes.len());
        for (&code, expected_code) in chunk.code.iter().zip(expected_codes) {
            assert_eq!(code, expected_code);
        }

        assert_eq!(chunk.lines.len(), expected_lines.len());
        for (&line, expected_line) in chunk.lines.iter().zip(expected_lines) {
            assert_eq!(line, expected_line);
        }

        assert_eq!(chunk.constants.len(), 2);
        for (constant, expected_constant) in chunk.constants.into_iter().zip(expected_constants) {
            assert_eq!(constant, expected_constant);
        }
    }

    #[test]
    fn it_compiles_a_greater_equal_expression() {
        let source = "1 >= 2;".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;

        let expected_codes = [
            OpCode::Constant as u8,
            0,
            OpCode::Constant as u8,
            1,
            OpCode::Less as u8,
            OpCode::Not as u8,
            OpCode::Pop as u8,
            OpCode::Nil as u8,
            OpCode::Return as u8,
        ];
        let expected_lines = [1; 9];
        let expected_constants = [Value::from(1.0), Value::from(2.0)];

        assert_eq!(chunk.code.len(), expected_codes.len());
        for (&code, expected_code) in chunk.code.iter().zip(expected_codes) {
            assert_eq!(code, expected_code);
        }

        assert_eq!(chunk.lines.len(), expected_lines.len());
        for (&line, expected_line) in chunk.lines.iter().zip(expected_lines) {
            assert_eq!(line, expected_line);
        }

        assert_eq!(chunk.constants.len(), 2);
        for (constant, expected_constant) in chunk.constants.into_iter().zip(expected_constants) {
            assert_eq!(constant, expected_constant);
        }
    }

    #[test]
    fn it_compiles_an_and_expression() {
        let source = "true and false;".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;

        let expected_codes = [
            OpCode::True as u8,
            OpCode::JumpIfFalse as u8,
            0,
            2,
            OpCode::Pop as u8,
            OpCode::False as u8,
            OpCode::Pop as u8,
            OpCode::Nil as u8,
            OpCode::Return as u8,
        ];
        let expected_lines = [1; 9];
        let expected_constants = [];

        assert_eq!(chunk.code.len(), expected_codes.len());
        for (&code, expected_code) in chunk.code.iter().zip(expected_codes) {
            assert_eq!(code, expected_code);
        }

        assert_eq!(chunk.lines.len(), expected_lines.len());
        for (&line, expected_line) in chunk.lines.iter().zip(expected_lines) {
            assert_eq!(line, expected_line);
        }

        assert_eq!(chunk.constants.len(), 0);
        for (constant, expected_constant) in chunk.constants.into_iter().zip(expected_constants) {
            assert_eq!(constant, expected_constant);
        }
    }

    #[test]
    fn it_compiles_an_or_expression() {
        let source = "true or false;".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;

        let expected_codes = [
            OpCode::True as u8,
            OpCode::JumpIfFalse as u8,
            0,
            3,
            OpCode::Jump as u8,
            0,
            2,
            OpCode::Pop as u8,
            OpCode::False as u8,
            OpCode::Pop as u8,
            OpCode::Nil as u8,
            OpCode::Return as u8,
        ];
        let expected_lines = [1; 12];
        let expected_constants = [];

        assert_eq!(chunk.code.len(), expected_codes.len());
        for (&code, expected_code) in chunk.code.iter().zip(expected_codes) {
            assert_eq!(code, expected_code);
        }

        assert_eq!(chunk.lines.len(), expected_lines.len());
        for (&line, expected_line) in chunk.lines.iter().zip(expected_lines) {
            assert_eq!(line, expected_line);
        }

        assert_eq!(chunk.constants.len(), 0);
        for (constant, expected_constant) in chunk.constants.into_iter().zip(expected_constants) {
            assert_eq!(constant, expected_constant);
        }
    }

    #[test]
    fn it_compiles_a_complex_expression() {
        let source = "!(1 + 2 * 3 / (4 - 5) > 6) or (7 <= 8 and 9 >= 10);".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;

        let expected_codes = [
            OpCode::Constant as u8,
            0,
            OpCode::Constant as u8,
            1,
            OpCode::Constant as u8,
            2,
            OpCode::Multiply as u8,
            OpCode::Constant as u8,
            3,
            OpCode::Constant as u8,
            4,
            OpCode::Subtract as u8,
            OpCode::Divide as u8,
            OpCode::Add as u8,
            OpCode::Constant as u8,
            5,
            OpCode::Greater as u8,
            OpCode::Not as u8,
            OpCode::JumpIfFalse as u8,
            0,
            3,
            OpCode::Jump as u8,
            0,
            17,
            OpCode::Pop as u8,
            OpCode::Constant as u8,
            6,
            OpCode::Constant as u8,
            7,
            OpCode::Greater as u8,
            OpCode::Not as u8,
            OpCode::JumpIfFalse as u8,
            0,
            7,
            OpCode::Pop as u8,
            OpCode::Constant as u8,
            8,
            OpCode::Constant as u8,
            9,
            OpCode::Less as u8,
            OpCode::Not as u8,
            OpCode::Pop as u8,
            OpCode::Nil as u8,
            OpCode::Return as u8,
        ];
        let expected_lines = [1; 44];
        let expected_constants = [
            Value::from(1.0),
            Value::from(2.0),
            Value::from(3.0),
            Value::from(4.0),
            Value::from(5.0),
            Value::from(6.0),
            Value::from(7.0),
            Value::from(8.0),
            Value::from(9.0),
            Value::from(10.0),
        ];

        assert_eq!(chunk.code.len(), expected_codes.len());
        for (&code, expected_code) in chunk.code.iter().zip(expected_codes) {
            assert_eq!(code, expected_code);
        }

        assert_eq!(chunk.lines.len(), expected_lines.len());
        for (&line, expected_line) in chunk.lines.iter().zip(expected_lines) {
            assert_eq!(line, expected_line);
        }

        assert_eq!(chunk.constants.len(), expected_constants.len());
        for (constant, expected_constant) in chunk.constants.into_iter().zip(expected_constants) {
            assert_eq!(constant, expected_constant);
        }
    }

    #[test]
    fn it_compiles_a_global_declaration() {
        let source = "var a;".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;

        let expected_codes = [
            OpCode::Nil as u8,
            OpCode::DefineGlobal as u8,
            0,
            OpCode::Nil as u8,
            OpCode::Return as u8,
        ];
        let expected_lines = [1; 5];
        let expected_constants = [Value::from("a")];

        assert_eq!(chunk.code.len(), expected_codes.len());
        for (&code, expected_code) in chunk.code.iter().zip(expected_codes) {
            assert_eq!(code, expected_code);
        }

        assert_eq!(chunk.lines.len(), expected_lines.len());
        for (&line, expected_line) in chunk.lines.iter().zip(expected_lines) {
            assert_eq!(line, expected_line);
        }

        assert_eq!(chunk.constants.len(), 1);
        for (constant, expected_constant) in chunk.constants.into_iter().zip(expected_constants) {
            assert_eq!(constant, expected_constant);
        }
    }

    #[test]
    fn it_compiles_a_global_definition() {
        let source = "var a = 1;".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;

        let expected_codes = [
            OpCode::Constant as u8,
            1,
            OpCode::DefineGlobal as u8,
            0,
            OpCode::Nil as u8,
            OpCode::Return as u8,
        ];
        let expected_lines = [1; 6];
        let expected_constants = [Value::from("a"), Value::from(1.0)];

        assert_eq!(chunk.code.len(), expected_codes.len());
        for (&code, expected_code) in chunk.code.iter().zip(expected_codes) {
            assert_eq!(code, expected_code);
        }

        assert_eq!(chunk.lines.len(), expected_lines.len());
        for (&line, expected_line) in chunk.lines.iter().zip(expected_lines) {
            assert_eq!(line, expected_line);
        }

        assert_eq!(chunk.constants.len(), 2);
        for (constant, expected_constant) in chunk.constants.into_iter().zip(expected_constants) {
            assert_eq!(constant, expected_constant);
        }
    }

    #[test]
    fn it_compiles_a_global_reference() {
        let source = "var a = 1; a;".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;

        let expected_codes = [
            OpCode::Constant as u8,
            1,
            OpCode::DefineGlobal as u8,
            0,
            OpCode::GetGlobal as u8,
            2,
            OpCode::Pop as u8,
            OpCode::Nil as u8,
            OpCode::Return as u8,
        ];
        let expected_lines = [1; 9];
        let expected_constants = [Value::from("a"), Value::from(1.0), Value::from("a")];

        assert_eq!(chunk.code.len(), expected_codes.len());
        for (&code, expected_code) in chunk.code.iter().zip(expected_codes) {
            assert_eq!(code, expected_code);
        }

        assert_eq!(chunk.lines.len(), expected_lines.len());
        for (&line, expected_line) in chunk.lines.iter().zip(expected_lines) {
            assert_eq!(line, expected_line);
        }

        assert_eq!(chunk.constants.len(), expected_constants.len());
        for (constant, expected_constant) in chunk.constants.into_iter().zip(expected_constants) {
            assert_eq!(constant, expected_constant);
        }
    }

    #[test]
    fn it_compiles_a_local_declaration() {
        let source = "{ var a; }".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;

        let expected_codes = [
            OpCode::Nil as u8,
            OpCode::Pop as u8,
            OpCode::Nil as u8,
            OpCode::Return as u8,
        ];

        let expected_lines = [1; 4];
        let expected_constants = [];

        assert_eq!(chunk.code.len(), expected_codes.len());
        for (&code, expected_code) in chunk.code.iter().zip(expected_codes) {
            assert_eq!(code, expected_code);
        }

        assert_eq!(chunk.lines.len(), expected_lines.len());
        for (&line, expected_line) in chunk.lines.iter().zip(expected_lines) {
            assert_eq!(line, expected_line);
        }

        assert_eq!(chunk.constants.len(), expected_constants.len());
        for (constant, expected_constant) in chunk.constants.into_iter().zip(expected_constants) {
            assert_eq!(constant, expected_constant);
        }
    }

    #[test]
    fn it_compiles_a_local_definition() {
        let source = "{ var a = 1; }".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;

        let expected_codes = [
            OpCode::Constant as u8,
            0,
            OpCode::Pop as u8,
            OpCode::Nil as u8,
            OpCode::Return as u8,
        ];

        let expected_lines = [1; 5];
        let expected_constants = [Value::from(1.0)];

        assert_eq!(chunk.code.len(), expected_codes.len());
        for (&code, expected_code) in chunk.code.iter().zip(expected_codes) {
            assert_eq!(code, expected_code);
        }

        assert_eq!(chunk.lines.len(), expected_lines.len());
        for (&line, expected_line) in chunk.lines.iter().zip(expected_lines) {
            assert_eq!(line, expected_line);
        }

        assert_eq!(chunk.constants.len(), expected_constants.len());
        for (constant, expected_constant) in chunk.constants.into_iter().zip(expected_constants) {
            assert_eq!(constant, expected_constant);
        }
    }

    #[test]
    fn it_compiles_a_local_reference() {
        let source = "{ var a = 1; a = a + 1; }".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;

        let expected_codes = [
            OpCode::Constant as u8,
            0,
            OpCode::GetLocal as u8,
            1,
            OpCode::Constant as u8,
            1,
            OpCode::Add as u8,
            OpCode::SetLocal as u8,
            1,
            OpCode::Pop as u8,
            OpCode::Pop as u8,
            OpCode::Nil as u8,
            OpCode::Return as u8,
        ];

        let expected_lines = [1; 13];
        let expected_constants = [Value::from(1.0), Value::from(1.0)];

        assert_eq!(chunk.code.len(), expected_codes.len());
        for (&code, expected_code) in chunk.code.iter().zip(expected_codes) {
            assert_eq!(code, expected_code);
        }

        assert_eq!(chunk.lines.len(), expected_lines.len());
        for (&line, expected_line) in chunk.lines.iter().zip(expected_lines) {
            assert_eq!(line, expected_line);
        }

        assert_eq!(chunk.constants.len(), expected_constants.len());
        for (constant, expected_constant) in chunk.constants.into_iter().zip(expected_constants) {
            assert_eq!(constant, expected_constant);
        }
    }

    #[test]
    fn it_compiles_a_global_reference_in_local_scope() {
        let source = "var a = 1; { a = a + 1; }".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;

        let expected_codes = [
            OpCode::Constant as u8,
            1,
            OpCode::DefineGlobal as u8,
            0,
            OpCode::GetGlobal as u8,
            3,
            OpCode::Constant as u8,
            4,
            OpCode::Add as u8,
            OpCode::SetGlobal as u8,
            2,
            OpCode::Pop as u8,
            OpCode::Nil as u8,
            OpCode::Return as u8,
        ];

        let expected_lines = [1; 14];
        let expected_constants = [
            Value::from("a"),
            Value::from(1.0),
            Value::from("a"),
            Value::from("a"),
            Value::from(1.0),
        ];

        assert_eq!(chunk.code.len(), expected_codes.len());
        for (&code, expected_code) in chunk.code.iter().zip(expected_codes) {
            assert_eq!(code, expected_code);
        }

        assert_eq!(chunk.lines.len(), expected_lines.len());
        for (&line, expected_line) in chunk.lines.iter().zip(expected_lines) {
            assert_eq!(line, expected_line);
        }

        assert_eq!(chunk.constants.len(), expected_constants.len());
        for (constant, expected_constant) in chunk.constants.into_iter().zip(expected_constants) {
            assert_eq!(constant, expected_constant);
        }
    }

    #[test]
    fn it_compiles_a_function_call() {
        let source = "fun foo(a, b) { return a + b; } foo(1, 2);".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;
        let expected_function_chunk = Chunk {
            code: vec![
                OpCode::GetLocal as u8,
                1,
                OpCode::GetLocal as u8,
                2,
                OpCode::Add as u8,
                OpCode::Return as u8,
                OpCode::Nil as u8,
                OpCode::Return as u8,
            ],
            lines: vec![1; 8],
            constants: vec![],
        };
        let expected_chunk = Chunk {
            code: vec![
                OpCode::Closure as u8,
                1,
                OpCode::DefineGlobal as u8,
                0,
                OpCode::GetGlobal as u8,
                2,
                OpCode::Constant as u8,
                3,
                OpCode::Constant as u8,
                4,
                OpCode::Call as u8,
                2,
                OpCode::Pop as u8,
                OpCode::Nil as u8,
                OpCode::Return as u8,
            ],
            lines: vec![1; 15],
            constants: vec![
                Value::from("foo"),
                Value::from(ObjFunction {
                    obj: Obj::default(),
                    arity: 2,
                    upvalue_count: 0,
                    chunk: expected_function_chunk,
                    name: Some(Rc::new(ObjString {
                        obj: Obj::default(),
                        chars: "foo".into(),
                    })),
                }),
                Value::from("foo"),
                Value::from(1.0),
                Value::from(2.0),
            ],
        };
        println!(
            "{}",
            chunk.constants.iter().fold(String::new(), |acc, value| {
                acc + &value.to_string() + ","
            })
        );
        assert_eq!(chunk, expected_chunk);
    }

    #[test]
    fn it_compiles_a_closure() {
        let source =
            "fun foo(a, b) { fun bar() { return a + b; } return bar(); } foo(1, 2);".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;
        let expected_bar_chunk = Chunk {
            code: vec![
                OpCode::GetUpvalue as u8,
                0,
                OpCode::GetUpvalue as u8,
                1,
                OpCode::Add as u8,
                OpCode::Return as u8,
                OpCode::Nil as u8,
                OpCode::Return as u8,
            ],
            lines: vec![1; 8],
            constants: vec![],
        };
        let expected_foo_chunk = Chunk {
            code: vec![
                OpCode::Closure as u8,
                0,
                1,
                1,
                1,
                2,
                OpCode::GetLocal as u8,
                3,
                OpCode::Call as u8,
                0,
                OpCode::Return as u8,
                OpCode::Nil as u8,
                OpCode::Return as u8,
            ],
            lines: vec![1; 13],
            constants: vec![Value::from(ObjFunction {
                obj: Obj::default(),
                arity: 0,
                upvalue_count: 2,
                chunk: expected_bar_chunk,
                name: Some(Rc::new(ObjString {
                    obj: Obj::default(),
                    chars: "bar".into(),
                })),
            })],
        };
        let expected_chunk = Chunk {
            code: vec![
                OpCode::Closure as u8,
                1,
                OpCode::DefineGlobal as u8,
                0,
                OpCode::GetGlobal as u8,
                2,
                OpCode::Constant as u8,
                3,
                OpCode::Constant as u8,
                4,
                OpCode::Call as u8,
                2,
                OpCode::Pop as u8,
                OpCode::Nil as u8,
                OpCode::Return as u8,
            ],
            lines: vec![1; 15],
            constants: vec![
                Value::from("foo"),
                Value::from(ObjFunction {
                    obj: Obj::default(),
                    arity: 2,
                    upvalue_count: 0,
                    chunk: expected_foo_chunk,
                    name: Some(Rc::new(ObjString {
                        obj: Obj::default(),
                        chars: "foo".into(),
                    })),
                }),
                Value::from("foo"),
                Value::from(1.0),
                Value::from(2.0),
            ],
        };
        let Value::Object(o) = &chunk.constants[1] else {
            panic!("Failed to read foo chunk.");
        };
        let Object::Function(foo) = &**o else {
            panic!("Failed to read foo chunk.");
        };

        let Value::Object(o) = &foo.chunk.constants[0] else {
            panic!("Failed to read bar chunk.");
        };
        let Object::Function(bar) = &**o else {
            panic!("Failed to read bar chunk.");
        };
        println!("{}", bar.chunk);
        println!("{}", foo.chunk);
        println!("{chunk}");
        assert_eq!(chunk, expected_chunk);
    }

    #[test]
    fn it_compiles_an_if_statement() {
        let source = "var a = 0; if (a > 0) { a = a + 1; } else { a = a - 1; }".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;
        let expected_chunk = Chunk {
            code: vec![
                OpCode::Constant as u8,
                1,
                OpCode::DefineGlobal as u8,
                0,
                OpCode::GetGlobal as u8,
                2,
                OpCode::Constant as u8,
                3,
                OpCode::Greater as u8,
                OpCode::JumpIfFalse as u8,
                0,
                12,
                OpCode::Pop as u8,
                OpCode::GetGlobal as u8,
                5,
                OpCode::Constant as u8,
                6,
                OpCode::Add as u8,
                OpCode::SetGlobal as u8,
                4,
                OpCode::Pop as u8,
                OpCode::Jump as u8,
                0,
                9,
                OpCode::Pop as u8,
                OpCode::GetGlobal as u8,
                8,
                OpCode::Constant as u8,
                9,
                OpCode::Subtract as u8,
                OpCode::SetGlobal as u8,
                7,
                OpCode::Pop as u8,
                OpCode::Nil as u8,
                OpCode::Return as u8,
            ],
            lines: vec![1; 35],
            constants: vec![
                Value::from("a"),
                Value::from(0.0),
                Value::from("a"),
                Value::from(0.0),
                Value::from("a"),
                Value::from("a"),
                Value::from(1.0),
                Value::from("a"),
                Value::from("a"),
                Value::from(1.0),
            ],
        };
        assert_eq!(chunk, expected_chunk);
    }

    #[test]
    fn it_compiles_a_for_loop() {
        let source = "for (var a = 0; a < 5; a = a + 1) { print \"for loop\"; }".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;
        let expected_chunk = Chunk {
            code: vec![
                OpCode::Constant as u8,
                0,
                OpCode::GetLocal as u8,
                1,
                OpCode::Constant as u8,
                1,
                OpCode::Less as u8,
                OpCode::JumpIfFalse as u8,
                0,
                21,
                OpCode::Pop as u8,
                OpCode::Jump as u8,
                0,
                11,
                OpCode::GetLocal as u8,
                1,
                OpCode::Constant as u8,
                2,
                OpCode::Add as u8,
                OpCode::SetLocal as u8,
                1,
                OpCode::Pop as u8,
                OpCode::Loop as u8,
                0,
                23,
                OpCode::Constant as u8,
                3,
                OpCode::Print as u8,
                OpCode::Loop as u8,
                0,
                17,
                OpCode::Pop as u8,
                OpCode::Pop as u8,
                OpCode::Nil as u8,
                OpCode::Return as u8,
            ],
            lines: vec![1; 35],
            constants: vec![
                Value::from(0.0),
                Value::from(5.0),
                Value::from(1.0),
                Value::from("for loop"),
            ],
        };
        assert_eq!(chunk, expected_chunk);
    }

    #[test]
    fn it_compiles_a_while_loop() {
        let source = "var a = 0; while (a < 5) { print \"while loop\"; a = a + 1; }".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;
        let expected_chunk = Chunk {
            code: vec![
                OpCode::Constant as u8,
                1,
                OpCode::DefineGlobal as u8,
                0,
                OpCode::GetGlobal as u8,
                2,
                OpCode::Constant as u8,
                3,
                OpCode::Less as u8,
                OpCode::JumpIfFalse as u8,
                0,
                15,
                OpCode::Pop as u8,
                OpCode::Constant as u8,
                4,
                OpCode::Print as u8,
                OpCode::GetGlobal as u8,
                6,
                OpCode::Constant as u8,
                7,
                OpCode::Add as u8,
                OpCode::SetGlobal as u8,
                5,
                OpCode::Pop as u8,
                OpCode::Loop as u8,
                0,
                23,
                OpCode::Pop as u8,
                OpCode::Nil as u8,
                OpCode::Return as u8,
            ],
            lines: vec![1; 30],
            constants: vec![
                Value::from("a"),
                Value::from(0.0),
                Value::from("a"),
                Value::from(5.0),
                Value::from("while loop"),
                Value::from("a"),
                Value::from("a"),
                Value::from(1.0),
            ],
        };
        assert_eq!(chunk, expected_chunk);
    }

    #[test]
    fn it_compiles_an_empty_class_declaration() {
        let source = "class TestClass {}".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;
        let expected_chunk = Chunk {
            code: vec![
                OpCode::Class as u8,
                0,
                OpCode::DefineGlobal as u8,
                0,
                OpCode::GetGlobal as u8,
                1,
                OpCode::Pop as u8,
                OpCode::Nil as u8,
                OpCode::Return as u8,
            ],
            lines: vec![1; 9],
            constants: vec![Value::from("TestClass"), Value::from("TestClass")],
        };
        assert_eq!(chunk, expected_chunk);
    }

    #[test]
    fn it_compiles_an_empty_class_initializer() {
        let source = "class TestClass { init() {} }".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;

        let expected_init_chunk = Chunk {
            code: vec![OpCode::GetLocal as u8, 0, OpCode::Return as u8],
            lines: vec![1; 3],
            constants: vec![],
        };
        let expected_chunk = Chunk {
            code: vec![
                OpCode::Class as u8,
                0,
                OpCode::DefineGlobal as u8,
                0,
                OpCode::GetGlobal as u8,
                1,
                OpCode::Closure as u8,
                3,
                OpCode::Method as u8,
                2,
                OpCode::Pop as u8,
                OpCode::Nil as u8,
                OpCode::Return as u8,
            ],
            lines: vec![1; 13],
            constants: vec![
                Value::from("TestClass"),
                Value::from("TestClass"),
                Value::from("init"),
                Value::from(ObjFunction {
                    obj: Obj::default(),
                    arity: 0,
                    upvalue_count: 0,
                    chunk: expected_init_chunk,
                    name: Some(Rc::new(ObjString {
                        obj: Obj::default(),
                        chars: "init".into(),
                    })),
                }),
            ],
        };
        assert_eq!(chunk, expected_chunk);
    }

    #[test]
    fn it_compiles_a_class_initializer() {
        let source = "class TestClass { init() { this.a = 1; this.b = this.a * 2; } }".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;

        let expected_init_chunk = Chunk {
            code: vec![
                OpCode::GetLocal as u8,
                0,
                OpCode::Constant as u8,
                1,
                OpCode::SetProperty as u8,
                0,
                OpCode::Pop as u8,
                OpCode::GetLocal as u8,
                0,
                OpCode::GetLocal as u8,
                0,
                OpCode::GetProperty as u8,
                3,
                OpCode::Constant as u8,
                4,
                OpCode::Multiply as u8,
                OpCode::SetProperty as u8,
                2,
                OpCode::Pop as u8,
                OpCode::GetLocal as u8,
                0,
                OpCode::Return as u8,
            ],
            lines: vec![1; 22],
            constants: vec![
                Value::from("a"),
                Value::from(1.0),
                Value::from("b"),
                Value::from("a"),
                Value::from(2.0),
            ],
        };
        let expected_chunk = Chunk {
            code: vec![
                OpCode::Class as u8,
                0,
                OpCode::DefineGlobal as u8,
                0,
                OpCode::GetGlobal as u8,
                1,
                OpCode::Closure as u8,
                3,
                OpCode::Method as u8,
                2,
                OpCode::Pop as u8,
                OpCode::Nil as u8,
                OpCode::Return as u8,
            ],
            lines: vec![1; 13],
            constants: vec![
                Value::from("TestClass"),
                Value::from("TestClass"),
                Value::from("init"),
                Value::from(ObjFunction {
                    obj: Obj::default(),
                    arity: 0,
                    upvalue_count: 0,
                    chunk: expected_init_chunk,
                    name: Some(Rc::new(ObjString {
                        obj: Obj::default(),
                        chars: "init".into(),
                    })),
                }),
            ],
        };
        let Value::Object(o) = &chunk.constants[3] else {
            panic!("Failed to get init chunk");
        };
        let Object::Function(init) = &**o else {
            panic!("Failed to get init chunk");
        };
        println!("{}", init.chunk);
        println!("{chunk}");
        assert_eq!(chunk, expected_chunk);
    }

    #[test]
    fn it_compiles_an_empty_method() {
        let source = "class TestClass { m() {} }".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;

        let expected_chunk = Chunk {
            code: vec![
                OpCode::Class as u8,
                0,
                OpCode::DefineGlobal as u8,
                0,
                OpCode::GetGlobal as u8,
                1,
                OpCode::Closure as u8,
                3,
                OpCode::Method as u8,
                2,
                OpCode::Pop as u8,
                OpCode::Nil as u8,
                OpCode::Return as u8,
            ],
            lines: vec![1; 13],
            constants: vec![
                Value::from("TestClass"),
                Value::from("TestClass"),
                Value::from("m"),
                Value::from(ObjFunction {
                    obj: Obj::default(),
                    arity: 0,
                    upvalue_count: 0,
                    name: Some(Rc::new(ObjString::from("m"))),
                    chunk: Chunk {
                        code: vec![OpCode::Nil as u8, OpCode::Return as u8],
                        lines: vec![1; 2],
                        constants: vec![],
                    },
                }),
            ],
        };
        assert_eq!(chunk, expected_chunk);
    }

    #[test]
    fn it_compiles_a_method_call() {
        let source = "class TestClass { init(a) {this.a = a;} m() { return this.a; } } var c = TestClass(); c.m();".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;
        let expected_init_chunk = Chunk {
            code: vec![
                OpCode::GetLocal as u8,
                0,
                OpCode::GetLocal as u8,
                1,
                OpCode::SetProperty as u8,
                0,
                OpCode::Pop as u8,
                OpCode::GetLocal as u8,
                0,
                OpCode::Return as u8,
            ],
            lines: vec![1; 10],
            constants: vec![Value::from("a")],
        };
        let expected_m_chunk = Chunk {
            code: vec![
                OpCode::GetLocal as u8,
                0,
                OpCode::GetProperty as u8,
                0,
                OpCode::Return as u8,
                OpCode::Nil as u8,
                OpCode::Return as u8,
            ],
            lines: vec![1; 7],
            constants: vec![Value::from("a")],
        };
        let expected_chunk = Chunk {
            code: vec![
                OpCode::Class as u8,
                0,
                OpCode::DefineGlobal as u8,
                0,
                OpCode::GetGlobal as u8,
                1,
                OpCode::Closure as u8,
                3,
                OpCode::Method as u8,
                2,
                OpCode::Closure as u8,
                5,
                OpCode::Method as u8,
                4,
                OpCode::Pop as u8,
                OpCode::GetGlobal as u8,
                7,
                OpCode::Call as u8,
                0,
                OpCode::DefineGlobal as u8,
                6,
                OpCode::GetGlobal as u8,
                8,
                OpCode::Invoke as u8,
                9,
                0,
                OpCode::Pop as u8,
                OpCode::Nil as u8,
                OpCode::Return as u8,
            ],
            lines: vec![1; 29],
            constants: vec![
                Value::from("TestClass"),
                Value::from("TestClass"),
                Value::from("init"),
                Value::from(ObjFunction {
                    obj: Obj::default(),
                    arity: 1,
                    upvalue_count: 0,
                    chunk: expected_init_chunk,
                    name: Some(Rc::new(ObjString::from("init"))),
                }),
                Value::from("m"),
                Value::from(ObjFunction {
                    obj: Obj::default(),
                    arity: 0,
                    upvalue_count: 0,
                    chunk: expected_m_chunk,
                    name: Some(Rc::new(ObjString::from("m"))),
                }),
                Value::from("c"),
                Value::from("TestClass"),
                Value::from("c"),
                Value::from("m"),
            ],
        };
        assert_eq!(chunk, expected_chunk);
    }

    #[test]
    fn it_compiles_a_sub_class() {
        let source = "class Parent {} class Child < Parent {}".into();
        let compiler = Compiler::new(source);
        let chunk = compiler.compile().unwrap().chunk;
        let expected_chunk = Chunk {
            code: vec![
                OpCode::Class as u8,
                0,
                OpCode::DefineGlobal as u8,
                0,
                OpCode::GetGlobal as u8,
                1,
                OpCode::Pop as u8,
                OpCode::Class as u8,
                2,
                OpCode::DefineGlobal as u8,
                2,
                OpCode::GetGlobal as u8,
                3,
                OpCode::GetGlobal as u8,
                4,
                OpCode::Inherit as u8,
                OpCode::GetGlobal as u8,
                5,
                OpCode::Pop as u8,
                OpCode::Pop as u8,
                OpCode::Nil as u8,
                OpCode::Return as u8,
            ],
            lines: vec![1; 22],
            constants: vec![
                Value::from("Parent"),
                Value::from("Parent"),
                Value::from("Child"),
                Value::from("Parent"),
                Value::from("Child"),
                Value::from("Child"),
            ],
        };
        assert_eq!(chunk, expected_chunk);
    }
}
