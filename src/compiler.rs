use crate::{
    chunk::OpCode,
    error::Error,
    object::{obj_function::ObjFunction, obj_string::ObjString, Obj, Object},
    scanner::Scanner,
    token::{Token, TokenType},
    value::Value,
};
use std::{array, iter::Peekable, rc::Rc};

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
pub struct Local {
    pub name: Token,
    pub depth: isize,
    pub is_captured: bool,
}

impl Default for Local {
    fn default() -> Self {
        Self {
            name: Token::default(),
            depth: -1,
            is_captured: false,
        }
    }
}

#[derive(Debug, Default)]
pub struct Upvalue {
    pub index: usize,
    pub is_local: bool,
}

#[derive(Debug)]
pub struct Compiler {
    enclosing: Option<Rc<Compiler>>,
    scanner: Peekable<Scanner>,
    had_error: bool,
    panic_mode: bool,
    current_function: ObjFunction,
    current_function_type: FunctionType,
    current_class_compiler: Option<Box<ClassCompiler>>,
    previous_token: Option<Token>,
    line: usize,
    scope_depth: usize,
    locals: [Local; u8::MAX as usize],
    local_count: usize,
    upvalues: [Upvalue; u8::MAX as usize],
    upvalue_count: usize,
}

impl Compiler {
    pub fn new(
        source: String,
        enclosing: Option<Rc<Compiler>>,
        function_type: FunctionType,
    ) -> Self {
        let scanner = Scanner::new(source).peekable();
        let upvalues = array::from_fn(|_| Upvalue::default());
        let mut locals = array::from_fn(|_| Local::default());
        let local = &mut locals[0];
        local.depth = 0;
        if function_type != FunctionType::Function {
            local.name = Token {
                kind: TokenType::Identifier,
                lexeme: "this".into(),
                line: 0,
            };
        } else {
            local.name = Token {
                kind: TokenType::Identifier,
                lexeme: "".into(),
                line: 0,
            }
        }
        Self {
            enclosing,
            scanner,
            current_function: ObjFunction::default(),
            current_function_type: function_type,
            current_class_compiler: None,
            line: 1,
            scope_depth: 0,
            had_error: false,
            panic_mode: false,
            previous_token: None,
            locals,
            local_count: 1,
            upvalues,
            upvalue_count: 0,
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

    fn emit_constant(&mut self, value: Value) {
        self.emit_opcode(OpCode::Constant);
        let constant = self.make_constant(value);
        self.emit_byte(constant);
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let constant = self.current_function.chunk.add_constant(value);
        if constant > u8::MAX as usize {
            self.error("Too many constants in one chunk.");
            return 0;
        }
        constant as u8
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

    fn end(mut self) -> ObjFunction {
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
        while self.local_count > 0
            && self.locals[self.local_count - 1].depth as usize > self.scope_depth
        {
            if self.locals[self.local_count - 1].is_captured {
                self.emit_opcode(OpCode::CloseUpvalue);
            } else {
                self.emit_opcode(OpCode::Pop);
            }
            self.local_count -= 1;
        }
    }

    fn mark_initialized(&mut self) {
        if self.scope_depth == 0 {
            return;
        }
        self.locals[self.local_count - 1].depth = self.scope_depth as isize;
    }

    fn resolve_local(&mut self, name: &Token) -> Option<usize> {
        for i in (0..self.local_count).rev() {
            let local = &self.locals[i];
            if Self::identifiers_equal(name, &local.name) {
                if local.depth == -1 {
                    self.error("can't read local variable in its own initializer.");
                }
                return Some(i);
            }
        }
        None
    }

    fn resolve_upvalue(&mut self, name: &Token) -> Option<usize> {
        let enclosing = Rc::get_mut(self.enclosing.as_mut()?)?;
        let local = enclosing.resolve_local(name);
        match local {
            Some(l) => {
                enclosing.locals[l].is_captured = true;
                return self.add_upvalue(l, true).into();
            }
            None => {
                if let Some(v) = enclosing.resolve_upvalue(name) {
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

    fn parse_variable(&mut self, error_message: &str) -> usize {
        todo!()
    }

    fn add_upvalue(&mut self, index: usize, is_local: bool) -> usize {
        let upvalue_count = self.current_function.upvalue_count;
        for i in 0..upvalue_count {
            let upvalue = &self.upvalues[i];
            if upvalue.index == index && upvalue.is_local == is_local {
                return i;
            }
        }

        if upvalue_count == u8::MAX as usize {
            self.error("Too many closure variables in function.");
        }

        self.upvalues[upvalue_count].is_local = is_local;
        self.upvalues[upvalue_count].index = index;
        self.current_function.upvalue_count += 1;
        self.current_function.upvalue_count
    }

    fn add_local(&mut self, name: Token) {
        if self.local_count == u8::MAX as usize {
            self.error("Too many local variables in function.");
            return;
        }

        let local = &mut self.locals[self.local_count];
        self.local_count += 1;
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

            self.named_variable(class_name.clone(), false);
            self.emit_byte(OpCode::Inherit as u8);
            self.current_class_compiler
                .as_deref_mut()
                .expect("ICE: Failed to get current class compiler.")
                .has_super_class = true;
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

    fn named_variable(&mut self, name: Token, can_assign: bool) {
        let get_op: OpCode;
        let set_op: OpCode;
        let mut arg = self.resolve_local(&name);
        if arg.is_some() {
            get_op = OpCode::GetLocal;
            set_op = OpCode::SetLocal;
        } else if ({
            arg = self.resolve_upvalue(&name);
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
        // let compiler = Compiler::new("".into(), Some(Rc::clone(self)), function_type);
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
