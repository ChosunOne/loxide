use std::fmt::{Display, Error};

use crate::{object::Object, value::Value};

#[derive(Debug, Default, PartialEq)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub lines: Vec<usize>,
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn write(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    fn simple_instruction(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        opcode: OpCode,
        offset: usize,
    ) -> Result<usize, Error> {
        writeln!(f, "{opcode}")?;
        Ok(offset + 1)
    }

    fn constant_instruction(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        opcode: OpCode,
        offset: usize,
    ) -> Result<usize, Error> {
        let constant = self.code[offset + 1] as usize;
        write!(f, "{opcode:<16}\t{constant:4}\t'")?;
        write!(f, "{}", self.constants[constant])?;
        writeln!(f, "'")?;
        Ok(offset + 2)
    }

    fn invoke_instruction(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        opcode: OpCode,
        offset: usize,
    ) -> Result<usize, Error> {
        let constant = self.code[offset + 1] as usize;
        let arg_count = self.code[offset + 2] as usize;
        write!(f, "{opcode:<4} ({arg_count} args)\t{constant:4}\t'")?;
        write!(f, "{}", self.constants[constant])?;
        writeln!(f, "'")?;
        Ok(offset + 3)
    }

    fn byte_instruction(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        opcode: OpCode,
        offset: usize,
    ) -> Result<usize, Error> {
        let slot = self.code[offset + 1] as usize;
        writeln!(f, "{opcode:<16}\t{slot:4}")?;
        Ok(offset + 2)
    }

    fn jump_instruction(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        opcode: OpCode,
        sign: i16,
        offset: usize,
    ) -> Result<usize, Error> {
        let mut jump = (self.code[offset + 1] as i16) << 8;
        jump |= self.code[offset + 2] as i16;
        writeln!(
            f,
            "{opcode:<16}\t{offset:4x} -> {:x}",
            offset as i16 + 3i16 + sign * jump
        )?;
        Ok(offset + 3)
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut offset = 0;
        while offset < self.code.len() {
            write!(f, "{offset:04x}\t")?;
            if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
                write!(f, "    |\t")?;
            } else {
                write!(f, "{:4}\t", self.lines[offset])?;
            }

            let instruction: OpCode = self.code[offset].into();
            offset = match instruction {
                o @ OpCode::Constant
                | o @ OpCode::GetGlobal
                | o @ OpCode::SetGlobal
                | o @ OpCode::DefineGlobal
                | o @ OpCode::GetProperty
                | o @ OpCode::SetProperty
                | o @ OpCode::GetSuper
                | o @ OpCode::Class
                | o @ OpCode::Method => self.constant_instruction(f, o, offset)?,
                o @ OpCode::Nil
                | o @ OpCode::True
                | o @ OpCode::False
                | o @ OpCode::Pop
                | o @ OpCode::Equal
                | o @ OpCode::Greater
                | o @ OpCode::Less
                | o @ OpCode::Add
                | o @ OpCode::Subtract
                | o @ OpCode::Multiply
                | o @ OpCode::Divide
                | o @ OpCode::Not
                | o @ OpCode::Negate
                | o @ OpCode::Print
                | o @ OpCode::CloseUpvalue
                | o @ OpCode::Return
                | o @ OpCode::Inherit
                | o @ OpCode::Unknown => self.simple_instruction(f, o, offset)?,
                o @ OpCode::GetLocal
                | o @ OpCode::SetLocal
                | o @ OpCode::GetUpvalue
                | o @ OpCode::SetUpvalue
                | o @ OpCode::Call => self.byte_instruction(f, o, offset)?,
                o @ OpCode::Jump | o @ OpCode::JumpIfFalse | o @ OpCode::Loop => {
                    self.jump_instruction(f, o, 1, offset)?
                }
                o @ OpCode::Invoke | o @ OpCode::SuperInvoke => {
                    self.invoke_instruction(f, o, offset)?
                }
                o @ OpCode::Closure => {
                    offset += 1;
                    let constant = self.code[offset] as usize;
                    offset += 1;
                    write!(f, "{:<16}\t{:4}\t", o, constant)?;
                    writeln!(f, "{}", self.constants[constant])?;
                    let function = match &self.constants[constant] {
                        Value::Object(bo) => match &**bo {
                            Object::Function(fun) => fun,
                            _ => panic!("Failed to get function from closure."),
                        },
                        _ => panic!("Failed to get function from closure."),
                    };

                    for _ in 0..function.upvalue_count {
                        let is_local = self.code[offset];
                        offset += 1;
                        let index = self.code[offset];
                        offset += 1;
                        write!(f, "{:04x}        |\t", offset - 2)?;
                        if is_local != 0 {
                            write!(f, "local")?;
                        } else {
                            write!(f, "upvalue")?;
                        }

                        writeln!(f, "{index}")?;
                    }

                    offset
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum OpCode {
    Constant = 0,
    Nil = 1,
    True = 2,
    False = 3,
    Pop = 4,
    GetLocal = 5,
    SetLocal = 6,
    GetGlobal = 7,
    SetGlobal = 8,
    DefineGlobal = 9,
    GetUpvalue = 10,
    SetUpvalue = 11,
    GetProperty = 12,
    SetProperty = 13,
    GetSuper = 14,
    Equal = 15,
    Greater = 16,
    Less = 17,
    Add = 18,
    Subtract = 19,
    Multiply = 20,
    Divide = 21,
    Not = 22,
    Negate = 23,
    Print = 24,
    Jump = 25,
    JumpIfFalse = 26,
    Loop = 27,
    Call = 28,
    Invoke = 29,
    SuperInvoke = 30,
    Closure = 31,
    CloseUpvalue = 32,
    Return = 33,
    Class = 34,
    Inherit = 35,
    Method = 36,
    Unknown = 255,
}

impl From<u8> for OpCode {
    fn from(value: u8) -> Self {
        match value {
            x if x == OpCode::Constant as u8 => OpCode::Constant,
            x if x == OpCode::Nil as u8 => OpCode::Nil,
            x if x == OpCode::True as u8 => OpCode::True,
            x if x == OpCode::False as u8 => OpCode::False,
            x if x == OpCode::Pop as u8 => OpCode::Pop,
            x if x == OpCode::GetLocal as u8 => OpCode::GetLocal,
            x if x == OpCode::SetLocal as u8 => OpCode::SetLocal,
            x if x == OpCode::GetGlobal as u8 => OpCode::GetGlobal,
            x if x == OpCode::SetGlobal as u8 => OpCode::SetGlobal,
            x if x == OpCode::DefineGlobal as u8 => OpCode::DefineGlobal,
            x if x == OpCode::GetUpvalue as u8 => OpCode::GetUpvalue,
            x if x == OpCode::SetUpvalue as u8 => OpCode::SetUpvalue,
            x if x == OpCode::GetProperty as u8 => OpCode::GetProperty,
            x if x == OpCode::SetProperty as u8 => OpCode::SetProperty,
            x if x == OpCode::GetSuper as u8 => OpCode::GetSuper,
            x if x == OpCode::Equal as u8 => OpCode::Equal,
            x if x == OpCode::Greater as u8 => OpCode::Greater,
            x if x == OpCode::Less as u8 => OpCode::Less,
            x if x == OpCode::Add as u8 => OpCode::Add,
            x if x == OpCode::Subtract as u8 => OpCode::Subtract,
            x if x == OpCode::Multiply as u8 => OpCode::Multiply,
            x if x == OpCode::Divide as u8 => OpCode::Divide,
            x if x == OpCode::Not as u8 => OpCode::Not,
            x if x == OpCode::Negate as u8 => OpCode::Negate,
            x if x == OpCode::Print as u8 => OpCode::Print,
            x if x == OpCode::Jump as u8 => OpCode::Jump,
            x if x == OpCode::JumpIfFalse as u8 => OpCode::JumpIfFalse,
            x if x == OpCode::Loop as u8 => OpCode::Loop,
            x if x == OpCode::Call as u8 => OpCode::Call,
            x if x == OpCode::Invoke as u8 => OpCode::Invoke,
            x if x == OpCode::SuperInvoke as u8 => OpCode::SuperInvoke,
            x if x == OpCode::Closure as u8 => OpCode::Closure,
            x if x == OpCode::CloseUpvalue as u8 => OpCode::CloseUpvalue,
            x if x == OpCode::Return as u8 => OpCode::Return,
            x if x == OpCode::Class as u8 => OpCode::Class,
            x if x == OpCode::Inherit as u8 => OpCode::Inherit,
            x if x == OpCode::Method as u8 => OpCode::Method,
            _ => OpCode::Unknown,
        }
    }
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Constant => write!(f, "OP_CONSTANT"),
            Self::Nil => write!(f, "OP_NIL"),
            Self::True => write!(f, "OP_TRUE"),
            Self::False => write!(f, "OP_FALSE"),
            Self::Pop => write!(f, "OP_POP"),
            Self::GetLocal => write!(f, "OP_GET_LOCAL"),
            Self::SetLocal => write!(f, "OP_SET_LOCAL"),
            Self::GetGlobal => write!(f, "OP_GET_GLOBAL"),
            Self::SetGlobal => write!(f, "OP_SET_GLOBAL"),
            Self::DefineGlobal => write!(f, "OP_DEFINE_GLOBAL"),
            Self::GetUpvalue => write!(f, "OP_GET_UPVALUE"),
            Self::SetUpvalue => write!(f, "OP_SET_UPVALUE"),
            Self::GetProperty => write!(f, "OP_GET_PROPERTY"),
            Self::SetProperty => write!(f, "OP_SET_PROPERTY"),
            Self::GetSuper => write!(f, "OP_GET_SUPER"),
            Self::Equal => write!(f, "OP_EQUAL"),
            Self::Greater => write!(f, "OP_GREATER"),
            Self::Less => write!(f, "OP_LESS"),
            Self::Add => write!(f, "OP_ADD"),
            Self::Subtract => write!(f, "OP_SUBTRACT"),
            Self::Multiply => write!(f, "OP_MULTIPLY"),
            Self::Divide => write!(f, "OP_DIVIDE"),
            Self::Not => write!(f, "OP_NOT"),
            Self::Negate => write!(f, "OP_NEGATE"),
            Self::Print => write!(f, "OP_PRINT"),
            Self::Jump => write!(f, "OP_JUMP"),
            Self::JumpIfFalse => write!(f, "OP_JUMP_IF_FALSE"),
            Self::Loop => write!(f, "OP_LOOP"),
            Self::Call => write!(f, "OP_CALL"),
            Self::Invoke => write!(f, "OP_INVOKE"),
            Self::SuperInvoke => write!(f, "OP_SUPER_INVOKE"),
            Self::Closure => write!(f, "OP_CLOSURE"),
            Self::CloseUpvalue => write!(f, "OP_CLOSE_UPVALUE"),
            Self::Return => write!(f, "OP_RETURN"),
            Self::Class => write!(f, "OP_CLASS"),
            Self::Inherit => write!(f, "OP_INHERIT"),
            Self::Method => write!(f, "OP_METHOD"),
            Self::Unknown => write!(f, "OP_UNKNOWN"),
        }
    }
}

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use crate::object::{obj_function::ObjFunction, obj_string::ObjString, Obj};

    use super::*;

    #[test]
    fn it_prints_constant_ops() {
        let mut chunk = Chunk::default();
        let constant_ops = [
            OpCode::Constant,
            OpCode::GetGlobal,
            OpCode::SetGlobal,
            OpCode::DefineGlobal,
            OpCode::GetProperty,
            OpCode::SetProperty,
            OpCode::GetSuper,
            OpCode::Class,
            OpCode::Method,
        ];

        for constant_op in constant_ops {
            chunk.write(constant_op as u8, 1);
            chunk.write(0, 1);
        }

        chunk.add_constant(Value::Number(1.0));
        let chunk_display = format!("{chunk}");
        assert_eq!(&chunk_display, "0000\t   1\tOP_CONSTANT\t   0\t'1'\n0002\t    |\tOP_GET_GLOBAL\t   0\t'1'\n0004\t    |\tOP_SET_GLOBAL\t   0\t'1'\n0006\t    |\tOP_DEFINE_GLOBAL\t   0\t'1'\n0008\t    |\tOP_GET_PROPERTY\t   0\t'1'\n000a\t    |\tOP_SET_PROPERTY\t   0\t'1'\n000c\t    |\tOP_GET_SUPER\t   0\t'1'\n000e\t    |\tOP_CLASS\t   0\t'1'\n0010\t    |\tOP_METHOD\t   0\t'1'\n");
    }

    #[test]
    fn it_prints_simple_ops() {
        let mut chunk = Chunk::default();
        let simple_ops = [
            OpCode::Nil,
            OpCode::True,
            OpCode::False,
            OpCode::Pop,
            OpCode::Equal,
            OpCode::Greater,
            OpCode::Less,
            OpCode::Add,
            OpCode::Subtract,
            OpCode::Multiply,
            OpCode::Divide,
            OpCode::Not,
            OpCode::Negate,
            OpCode::Print,
            OpCode::CloseUpvalue,
            OpCode::Return,
            OpCode::Inherit,
            OpCode::Unknown,
        ];

        for simple_op in simple_ops {
            chunk.write(simple_op as u8, 1);
        }

        let chunk_display = format!("{chunk}");
        let expected_chunk_display = "0000\t   1\tOP_NIL\n0001\t    |\tOP_TRUE\n0002\t    |\tOP_FALSE\n0003\t    |\tOP_POP\n0004\t    |\tOP_EQUAL\n0005\t    |\tOP_GREATER\n0006\t    |\tOP_LESS\n0007\t    |\tOP_ADD\n0008\t    |\tOP_SUBTRACT\n0009\t    |\tOP_MULTIPLY\n000a\t    |\tOP_DIVIDE\n000b\t    |\tOP_NOT\n000c\t    |\tOP_NEGATE\n000d\t    |\tOP_PRINT\n000e\t    |\tOP_CLOSE_UPVALUE\n000f\t    |\tOP_RETURN\n0010\t    |\tOP_INHERIT\n0011\t    |\tOP_UNKNOWN\n";
        assert_eq!(&chunk_display, expected_chunk_display);
    }

    #[test]
    fn it_prints_byte_ops() {
        let mut chunk = Chunk::default();
        let byte_ops = [
            OpCode::GetLocal,
            OpCode::SetLocal,
            OpCode::GetUpvalue,
            OpCode::SetUpvalue,
            OpCode::Call,
        ];

        for (slot, &byte_op) in byte_ops.iter().enumerate() {
            chunk.write(byte_op as u8, 1);
            chunk.write(slot as u8, 1);
        }
        let chunk_display = format!("{chunk}");
        assert_eq!(chunk_display, "0000\t   1\tOP_GET_LOCAL\t   0\n0002\t    |\tOP_SET_LOCAL\t   1\n0004\t    |\tOP_GET_UPVALUE\t   2\n0006\t    |\tOP_SET_UPVALUE\t   3\n0008\t    |\tOP_CALL\t   4\n");
    }

    #[test]
    fn it_prints_jump_ops() {
        let mut chunk = Chunk::default();
        let jump_ops = [OpCode::Jump, OpCode::JumpIfFalse, OpCode::Loop];

        for jump_op in jump_ops {
            chunk.write(jump_op as u8, 1);
            chunk.write(0xffu8, 1);
            chunk.write(0xffu8, 1);
        }

        let chunk_display = format!("{chunk}");
        print!("{chunk_display}");
        assert_eq!(&chunk_display, "0000\t   1\tOP_JUMP\t   0 -> 2\n0003\t    |\tOP_JUMP_IF_FALSE\t   3 -> 5\n0006\t    |\tOP_LOOP\t   6 -> 8\n");
    }

    #[test]
    fn it_prints_invoke_ops() {
        let mut chunk = Chunk::default();
        let invoke_ops = [OpCode::Invoke, OpCode::SuperInvoke];

        for invoke_op in invoke_ops {
            chunk.add_constant(Value::Number(0.0));
            chunk.write(invoke_op as u8, 1);
            chunk.write(0u8, 1);
            chunk.write(0u8, 1);
        }

        let chunk_display = format!("{chunk}");
        assert_eq!(&chunk_display, "0000\t   1\tOP_INVOKE (0 args)\t   0\t'0'\n0003\t    |\tOP_SUPER_INVOKE (0 args)\t   0\t'0'\n");
    }

    #[test]
    fn it_prints_closure_ops() {
        let mut chunk = Chunk::default();
        let function_name = Rc::new(ObjString {
            obj: Obj::default(),
            chars: "closure".into(),
            hash: 12345,
        });
        let function = ObjFunction {
            obj: Obj::default(),
            arity: 0,
            name: Some(function_name),
            chunk: Chunk::default(),
            upvalue_count: 2,
        };
        chunk.add_constant(Value::Object(Box::new(Object::Function(function))));
        chunk.write(OpCode::Closure as u8, 1);
        chunk.write(0, 1);
        chunk.write(1, 1); // local value
        chunk.write(1, 1); // index
        chunk.write(0, 1); // upvalue
        chunk.write(2, 1); // index

        let chunk_display = format!("{chunk}");
        assert_eq!(&chunk_display, "0000\t   1\tOP_CLOSURE\t   0\t<fn closure>\n0002        |\tlocal1\n0004        |\tupvalue2\n");
    }
}
