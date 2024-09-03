use crate::error::Error;

pub struct VM {}

impl VM {
    pub fn new() -> Self {
        Self {}
    }

    pub fn interpret(&mut self, source: &str) -> Result<(), Error> {
        #[cfg(feature = "debug")]
        println!("========== CODE ==========");

        todo!();
    }
}
