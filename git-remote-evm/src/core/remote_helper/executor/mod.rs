mod background;
mod browser;
mod link_opener;
mod mock;

use crate::core::{
    hash::Hash,
    object::Object,
    reference::Reference,
    remote_helper::{config::Wallet, error::RemoteHelperError},
};
use async_trait::async_trait;
use background::Background;
// use browser::Browser;
// use link_opener::browser::BrowserLinkOpener;ƒ

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
    wallet: Wallet,
) -> Result<Box<dyn Executor>, RemoteHelperError> {
    match wallet {
        // true => Ok(Box::new(Browser::new(Box::new(BrowserLinkOpener))?)),
        Wallet::Browser => todo!(),
        _ => Ok(Box::new(Background::new(wallet, rpc, [0; 20]).await?)),
    }
}
