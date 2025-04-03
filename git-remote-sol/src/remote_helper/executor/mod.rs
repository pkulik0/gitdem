use std::error::Error;

mod browser;
use browser::{Browser, BrowserLinkOpener};
mod background;
use background::Background;
use super::Wallet;

#[cfg(test)]
mod mock;
mod assets;

pub struct Transaction;

pub trait Executor {
    fn execute(&self, transaction: Transaction) -> Result<(), Box<dyn Error>>;
}

pub fn create_executor(wallet: &dyn Wallet) -> Box<dyn Executor> {
    match wallet.is_extension() {
        true => Box::new(Browser::new(Box::new(BrowserLinkOpener)).unwrap()), // TODO: handle error
        false => Box::new(Background::new()),
    }
}
