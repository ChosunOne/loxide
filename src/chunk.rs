use crate::value::Value;

#[derive(Debug, PartialEq)]
pub struct Chunk<'a> {
    pub code: Vec<u8>,
    pub lines: Vec<usize>,
    pub constants: Vec<Value<'a>>,
}
