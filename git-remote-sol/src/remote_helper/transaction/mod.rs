use std::error::Error;
use super::solana::Wallet;

mod browser;
use browser::Browser;
mod background;
use background::Background;

#[cfg(test)]
mod tests;

pub struct Transaction;

pub trait Executor {
    fn execute(&self, transaction: Transaction) -> Result<(), Box<dyn Error>>;
}

pub fn create_executor(wallet: &dyn Wallet) -> Box<dyn Executor> {
    match wallet.is_extension() {
        true => Box::new(Browser::new().unwrap()), // TODO: handle error
        false => Box::new(Background::new()),
    }
}
