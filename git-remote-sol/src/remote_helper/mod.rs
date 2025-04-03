use std::error::Error;

use reference::{Reference, ReferencePush};

pub mod solana;
pub mod reference;
pub mod hash;
mod executor;

#[cfg(any(test, feature = "mock"))]
pub mod mock;

#[derive(Debug, PartialEq)]
pub enum RemoteHelperError {
    InvalidHash(String),
}

impl Error for RemoteHelperError {}

impl std::fmt::Display for RemoteHelperError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidHash(hash) => write!(f, "invalid hash: {}", hash),
        }
    }
}

pub trait RemoteHelper {
    fn capabilities(&self) -> Vec<&'static str>;
    fn list(&self, is_for_push: bool) -> Result<Vec<Reference>, RemoteHelperError>;
    fn fetch(&self, reference: &Reference) -> Result<(), RemoteHelperError>;
    fn push(&self, reference: &ReferencePush) -> Result<(), RemoteHelperError>;
}

pub trait Wallet {
    fn is_extension(&self) -> bool;
}
