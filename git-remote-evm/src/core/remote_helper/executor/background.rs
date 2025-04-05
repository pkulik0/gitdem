use super::Executor;
use super::Transaction;
use std::error::Error;

pub struct Background {}

impl Background {
    pub fn new() -> Self {
        Self {}
    }
}

impl Executor for Background {
    fn execute(&self, transaction: Transaction) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
