pub use crate::token::Token;

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
