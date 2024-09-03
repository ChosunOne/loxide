use crate::scanner::Scanner;

pub struct Compiler<'a> {
    scanner: Scanner<'a>,
}

impl<'a> Compiler<'a> {
    pub fn new(scanner: Scanner<'a>) -> Self {
        Self { scanner }
    }
}
