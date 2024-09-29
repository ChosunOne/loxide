use crate::{chunk::Chunk, value::ConstantValue};
use std::{cell::RefCell, fmt::Display, rc::Rc};

use super::HeapSize;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ObjFunction {
    pub arity: usize,
    pub upvalue_count: usize,
    pub chunk: Rc<RefCell<Chunk>>,
    pub name: Option<String>,
}

impl HeapSize for ObjFunction {
    fn size(&self) -> usize {
        size_of::<usize>() * 2
            + self.chunk.borrow().code.len()
            + self.chunk.borrow().lines.len() * size_of::<usize>()
            + self
                .chunk
                .borrow()
                .constants
                .iter()
                .map(|x| match &**x {
                    ConstantValue::Number(_) => size_of::<f64>(),
                    ConstantValue::String(s) => s.chars.len(),
                    ConstantValue::Function(obj_function) => obj_function.size(),
                })
                .sum::<usize>()
    }
}

impl Display for ObjFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.name {
            None => write!(f, "<script>"),
            Some(s) => write!(f, "<fn {s}>"),
        }
    }
}
