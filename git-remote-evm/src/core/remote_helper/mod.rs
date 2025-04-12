pub mod config;
pub mod error;
pub mod evm;
pub mod executor;

use crate::core::hash::Hash;
use crate::core::reference::{Push, Reference};
use error::RemoteHelperError;
use mockall::automock;

#[automock]
pub trait RemoteHelper {
    fn capabilities(&self) -> Vec<&'static str>;
    fn list(&self, is_for_push: bool) -> Result<Vec<Reference>, RemoteHelperError>;
    fn fetch(&self, hash: Hash) -> Result<(), RemoteHelperError>;
    fn push(&self, pushes: Vec<Push>) -> Result<(), RemoteHelperError>;
}
