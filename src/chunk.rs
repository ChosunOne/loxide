use crate::value::Value;

pub struct Chunk<'a> {
    code: Vec<u8>,
    lines: Vec<usize>,
    constants: Vec<Value<'a>>,
}
