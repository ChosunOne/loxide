use std::fmt::Display;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error {
    Runtime,
    Compile,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Runtime => f.write_str("Runtime error"),
            Self::Compile => f.write_str("Compile error"),
        }
    }
}

impl std::error::Error for Error {}
