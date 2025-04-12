pub mod config;
pub mod error;
pub mod evm;
mod executor;
#[cfg(any(test, feature = "mock"))]
pub mod mock;

use crate::core::hash::Hash;
use crate::core::reference::{Reference, Push};
use error::RemoteHelperError;

pub trait RemoteHelper {
    fn capabilities(&self) -> Vec<&'static str>;
    fn list(&self, is_for_push: bool) -> Result<Vec<Reference>, RemoteHelperError>;
    fn fetch(&self, hash: Hash) -> Result<(), RemoteHelperError>;
    fn push(&self, pushes: Vec<Push>) -> Result<(), RemoteHelperError>;
}

pub trait Wallet {
    fn is_extension(&self) -> bool;
}
