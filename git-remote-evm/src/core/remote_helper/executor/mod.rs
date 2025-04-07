mod background;
mod browser;
mod link_opener;

use super::{config::EvmWallet, Wallet};
use crate::core::{reference::Reference, remote_helper::error::RemoteHelperError};
use background::Background;
// use browser::Browser;
// use link_opener::browser::BrowserLinkOpener;
use async_trait::async_trait;

#[async_trait]
pub trait Executor {
    async fn list(&self) -> Result<Vec<Reference>, RemoteHelperError>;
}

pub async fn create_executor(rpc: &str, wallet_type: EvmWallet) -> Result<Box<dyn Executor>, RemoteHelperError> {
    match wallet_type.is_extension() {
        // true => Ok(Box::new(Browser::new(Box::new(BrowserLinkOpener))?)),
        true => todo!(),
        false => Ok(Box::new(Background::new(wallet_type, rpc, [0; 20]).await?)),
    }
}
