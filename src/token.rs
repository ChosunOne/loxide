#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Token {
    pub kind: TokenType,
    pub lexeme: String,
    pub line: usize,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub enum TokenType {
    // Single character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    // One or two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // Literals
    Identifier,
    String,
    Number,
    // Keywords
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    #[default]
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Error,
    Eof,
}

pub struct InfixBindingPower {
    pub left_binding_power: u8,
    pub right_binding_power: u8,
}

impl From<(u8, u8)> for InfixBindingPower {
    fn from(value: (u8, u8)) -> Self {
        Self {
            left_binding_power: value.0,
            right_binding_power: value.1,
        }
    }
}

pub struct PrefixBindingPower {
    pub binding_power: u8,
}

impl PartialEq<InfixBindingPower> for PrefixBindingPower {
    fn eq(&self, other: &InfixBindingPower) -> bool {
        self.binding_power == other.right_binding_power
    }
}

impl PartialOrd<InfixBindingPower> for PrefixBindingPower {
    fn partial_cmp(&self, other: &InfixBindingPower) -> Option<std::cmp::Ordering> {
        self.binding_power.partial_cmp(&other.right_binding_power)
    }
}

impl From<u8> for PrefixBindingPower {
    fn from(value: u8) -> Self {
        Self {
            binding_power: value,
        }
    }
}

pub struct PostfixBindingPower {
    pub binding_power: u8,
}

impl PartialEq<InfixBindingPower> for PostfixBindingPower {
    fn eq(&self, other: &InfixBindingPower) -> bool {
        self.binding_power == other.left_binding_power
    }
}

impl PartialOrd<InfixBindingPower> for PostfixBindingPower {
    fn partial_cmp(&self, other: &InfixBindingPower) -> Option<std::cmp::Ordering> {
        self.binding_power.partial_cmp(&other.left_binding_power)
    }
}

impl From<u8> for PostfixBindingPower {
    fn from(value: u8) -> Self {
        Self {
            binding_power: value,
        }
    }
}

impl TokenType {
    pub fn infix_binding_power(&self) -> Option<InfixBindingPower> {
        match self {
            // `=` is right associative
            Self::Equal => Some((2, 1).into()),
            Self::And | Self::Or => Some((3, 4).into()),
            Self::EqualEqual | Self::BangEqual => Some((5, 6).into()),
            Self::Less | Self::LessEqual | Self::Greater | Self::GreaterEqual => {
                Some((7, 8).into())
            }
            Self::Plus | Self::Minus => Some((9, 10).into()),
            Self::Star | Self::Slash => Some((11, 12).into()),
            // `.` and `(` are right associative
            Self::Dot | Self::LeftParen => Some((15, 14).into()),
            _ => None,
        }
    }

    pub fn prefix_binding_power(&self) -> Option<PrefixBindingPower> {
        match self {
            Self::Bang | Self::Minus => Some(13.into()),
            _ => None,
        }
    }

    pub fn postfix_binding_power(&self) -> Option<PostfixBindingPower> {
        match self {
            _ => None,
        }
    }
}
