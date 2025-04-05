mod background;
mod browser;
mod contract;
mod link_opener;

use super::Wallet;
use crate::core::remote_helper::error::RemoteHelperError;
use background::Background;
use browser::Browser;
use link_opener::browser::BrowserLinkOpener;

pub struct Transaction;

pub trait Executor {
    fn execute(&self, transaction: Transaction) -> Result<(), RemoteHelperError>;
}

pub fn create_executor(wallet: &dyn Wallet) -> Result<Box<dyn Executor>, RemoteHelperError> {
    match wallet.is_extension() {
        true => Ok(Box::new(Browser::new(Box::new(BrowserLinkOpener))?)),
        false => Ok(Box::new(Background::new())),
    }
}
