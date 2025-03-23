use super::Executor;
use super::Transaction;
use std::error::Error;

pub struct Browser{}

impl Browser {
    pub fn new() -> Self {
        Self{}
    }
}

impl Executor for Browser {
    fn execute(&self, transaction: Transaction) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
