use crate::{chunk::Chunk, value::ConstantValue};
use std::fmt::Display;

use super::HeapSize;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ObjFunction {
    pub arity: usize,
    pub upvalue_count: usize,
    pub chunk: Chunk,
    pub name: Option<String>,
}

impl HeapSize for ObjFunction {
    fn size(&self) -> usize {
        size_of::<usize>() * 2
            + self.chunk.code.len()
            + self.chunk.lines.len() * size_of::<usize>()
            + self
                .chunk
                .constants
                .iter()
                .map(|x| match x {
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
