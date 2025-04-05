pub mod config;
mod contract;
pub mod error;
pub mod evm;
mod executor;
#[cfg(any(test, feature = "mock"))]
pub mod mock;

use crate::core::reference::{Reference, ReferencePush};
use error::RemoteHelperError;

pub trait RemoteHelper {
    fn capabilities(&self) -> Vec<&'static str>;
    fn list(&self, is_for_push: bool) -> Result<Vec<Reference>, RemoteHelperError>;
    fn fetch(&self, reference: &Reference) -> Result<(), RemoteHelperError>;
    fn push(&self, reference: &ReferencePush) -> Result<(), RemoteHelperError>;
}

pub trait Wallet {
    fn is_extension(&self) -> bool;
}
