mod background;
mod browser;
mod link_opener;
mod mock;

use super::{Wallet, config::EvmWallet};
use crate::core::{
    hash::Hash, object::Object, reference::Reference, remote_helper::error::RemoteHelperError,
};
use background::Background;
// use browser::Browser;
// use link_opener::browser::BrowserLinkOpener;
use async_trait::async_trait;
use mock::Mock;

#[async_trait]
pub trait Executor {
    async fn list(&self) -> Result<Vec<Reference>, RemoteHelperError>;
    async fn push(
        &self,
        objects: Vec<Object>,
        refs: Vec<Reference>,
    ) -> Result<(), RemoteHelperError>;
    async fn fetch(&self, hash: Hash) -> Result<Object, RemoteHelperError>;
    async fn resolve_references(&self, names: Vec<String>) -> Result<Vec<Hash>, RemoteHelperError>;
    async fn list_objects(&self) -> Result<Vec<Hash>, RemoteHelperError>;
}

pub async fn create_executor(
    rpc: &str,
    wallet_type: EvmWallet,
) -> Result<Box<dyn Executor>, RemoteHelperError> {
    #[cfg(test)]
    return Ok(Box::new(Mock::new()));

    match wallet_type.is_extension() {
        // true => Ok(Box::new(Browser::new(Box::new(BrowserLinkOpener))?)),
        true => todo!(),
        false => Ok(Box::new(Background::new(wallet_type, rpc, [0; 20]).await?)),
    }
}
