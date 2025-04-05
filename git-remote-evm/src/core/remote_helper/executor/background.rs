use super::Executor;
use super::Transaction;
use crate::core::remote_helper::error::RemoteHelperError;

pub struct Background {}

impl Background {
    pub fn new() -> Self {
        Self {}
    }
}

impl Executor for Background {
    fn execute(&self, transaction: Transaction) -> Result<(), RemoteHelperError> {
        Ok(())
    }
}
