use crate::{error::Error, token::TokenType};

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum BindingPower {
    Group,
    // Assignment is right associative
    AssignmentRight,
    AssignmentLeft,
    LogicalLeft,
    LogicalRight,
    EqualityLeft,
    EqualityRight,
    ComparisonLeft,
    ComparisonRight,
    TermLeft,
    TermRight,
    FactorLeft,
    FactorRight,
    Unary,
    // Call is right associative
    CallRight,
    CallLeft,
}

pub struct InfixBindingPower {
    pub left_binding_power: BindingPower,
    pub right_binding_power: BindingPower,
}

impl TryFrom<TokenType> for InfixBindingPower {
    type Error = Error;

    fn try_from(value: TokenType) -> Result<Self, Self::Error> {
        match value {
            TokenType::Equal => {
                Ok((BindingPower::AssignmentLeft, BindingPower::AssignmentRight).into())
            }
            TokenType::And | TokenType::Or => {
                Ok((BindingPower::LogicalLeft, BindingPower::LogicalRight).into())
            }
            TokenType::EqualEqual | TokenType::BangEqual => {
                Ok((BindingPower::EqualityLeft, BindingPower::EqualityRight).into())
            }
            TokenType::Less
            | TokenType::LessEqual
            | TokenType::Greater
            | TokenType::GreaterEqual => {
                Ok((BindingPower::ComparisonLeft, BindingPower::ComparisonRight).into())
            }
            TokenType::Plus | TokenType::Minus => {
                Ok((BindingPower::TermLeft, BindingPower::TermRight).into())
            }
            TokenType::Star | TokenType::Slash => {
                Ok((BindingPower::FactorLeft, BindingPower::FactorRight).into())
            }
            TokenType::Dot | TokenType::LeftParen => {
                Ok((BindingPower::CallLeft, BindingPower::CallRight).into())
            }
            _ => Err(Error::Compile),
        }
    }
}

impl From<(BindingPower, BindingPower)> for InfixBindingPower {
    fn from(value: (BindingPower, BindingPower)) -> Self {
        Self {
            left_binding_power: value.0,
            right_binding_power: value.1,
        }
    }
}

impl PartialEq<BindingPower> for InfixBindingPower {
    fn eq(&self, other: &BindingPower) -> bool {
        self.left_binding_power.eq(other)
    }
}

impl PartialOrd<BindingPower> for InfixBindingPower {
    fn partial_cmp(&self, other: &BindingPower) -> Option<std::cmp::Ordering> {
        self.left_binding_power.partial_cmp(other)
    }
}

pub struct PrefixBindingPower {
    pub binding_power: BindingPower,
}

impl TryFrom<TokenType> for PrefixBindingPower {
    type Error = Error;

    fn try_from(value: TokenType) -> Result<Self, Self::Error> {
        match value {
            TokenType::LeftParen => Ok(BindingPower::Group.into()),
            TokenType::Bang | TokenType::Minus => Ok(BindingPower::Unary.into()),
            _ => Err(Error::Compile),
        }
    }
}

impl PartialEq<InfixBindingPower> for PrefixBindingPower {
    fn eq(&self, other: &InfixBindingPower) -> bool {
        self.binding_power == other.right_binding_power
    }
}

impl PartialEq<BindingPower> for PrefixBindingPower {
    fn eq(&self, other: &BindingPower) -> bool {
        self.binding_power.eq(other)
    }
}

impl PartialOrd<BindingPower> for PrefixBindingPower {
    fn partial_cmp(&self, other: &BindingPower) -> Option<std::cmp::Ordering> {
        self.binding_power.partial_cmp(other)
    }
}

impl PartialOrd<InfixBindingPower> for PrefixBindingPower {
    fn partial_cmp(&self, other: &InfixBindingPower) -> Option<std::cmp::Ordering> {
        self.binding_power.partial_cmp(&other.right_binding_power)
    }
}

impl From<BindingPower> for PrefixBindingPower {
    fn from(value: BindingPower) -> Self {
        Self {
            binding_power: value,
        }
    }
}

pub struct PostfixBindingPower {
    pub binding_power: BindingPower,
}

impl TryFrom<TokenType> for PostfixBindingPower {
    type Error = Error;

    fn try_from(_value: TokenType) -> Result<Self, Self::Error> {
        // TODO: Add a match here if postfix operators are added
        Err(Error::Compile)
    }
}

impl PartialEq<BindingPower> for PostfixBindingPower {
    fn eq(&self, other: &BindingPower) -> bool {
        self.binding_power.eq(other)
    }
}

impl PartialEq<InfixBindingPower> for PostfixBindingPower {
    fn eq(&self, other: &InfixBindingPower) -> bool {
        self.binding_power.eq(&other.left_binding_power)
    }
}

impl PartialOrd<BindingPower> for PostfixBindingPower {
    fn partial_cmp(&self, other: &BindingPower) -> Option<std::cmp::Ordering> {
        self.binding_power.partial_cmp(other)
    }
}

impl PartialOrd<InfixBindingPower> for PostfixBindingPower {
    fn partial_cmp(&self, other: &InfixBindingPower) -> Option<std::cmp::Ordering> {
        self.binding_power.partial_cmp(&other.left_binding_power)
    }
}

impl From<BindingPower> for PostfixBindingPower {
    fn from(value: BindingPower) -> Self {
        Self {
            binding_power: value,
        }
    }
}
