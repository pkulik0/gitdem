mod background;
mod browser;
mod link_opener;

use std::error::Error;

use super::Wallet;
use background::Background;
use browser::Browser;
use link_opener::browser::BrowserLinkOpener;

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
