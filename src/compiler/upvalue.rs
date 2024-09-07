#[derive(Debug, Default)]
pub struct Upvalue {
    pub index: usize,
    pub is_local: bool,
}
