use std::error::Error;

use reference::Reference;

pub mod solana;
pub mod reference;
mod transaction;

#[cfg(any(test, feature = "mock"))]
pub mod mock;
#[cfg(test)]
pub mod tests;

#[derive(Debug)]
pub enum RemoteHelperError {}

impl Error for RemoteHelperError {}

impl std::fmt::Display for RemoteHelperError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

pub trait RemoteHelper {
    fn capabilities(&self) -> Vec<&'static str>;
    fn list(&self) -> Result<Vec<Reference>, RemoteHelperError>;
    fn fetch(&self, refs: &[Reference]) -> Result<(), RemoteHelperError>;
}

pub trait Wallet {
    fn is_extension(&self) -> bool;
}
