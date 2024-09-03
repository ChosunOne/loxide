#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TokenData {
    pub lexeme: String,
    pub line: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Token {
    // Single character tokens
    LeftParen(TokenData),
    RightParen(TokenData),
    LeftBrace(TokenData),
    RightBrace(TokenData),
    Comma(TokenData),
    Dot(TokenData),
    Minus(TokenData),
    Plus(TokenData),
    Semicolon(TokenData),
    Slash(TokenData),
    Star(TokenData),
    // One or two character tokens
    Bang(TokenData),
    BangEqual(TokenData),
    Equal(TokenData),
    EqualEqual(TokenData),
    Greater(TokenData),
    GreaterEqual(TokenData),
    Less(TokenData),
    LessEqual(TokenData),
    // Literals
    Identifier(TokenData),
    String(TokenData),
    Number(TokenData),
    // Keywords
    And(TokenData),
    Class(TokenData),
    Else(TokenData),
    False(TokenData),
    For(TokenData),
    Fun(TokenData),
    If(TokenData),
    Nil(TokenData),
    Or(TokenData),
    Print(TokenData),
    Return(TokenData),
    Super(TokenData),
    This(TokenData),
    True(TokenData),
    Var(TokenData),
    While(TokenData),
    Error(TokenData),
    Eof(TokenData),
}
