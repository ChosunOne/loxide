use crate::scanner::Scanner;
use std::iter::Peekable;

pub struct Compiler<'a> {
    scanner: Peekable<Scanner<'a>>,
}

impl<'a> Compiler<'a> {
    pub fn new(scanner: Scanner<'a>) -> Self {
        Self {
            scanner: scanner.peekable(),
        }
    }
}
