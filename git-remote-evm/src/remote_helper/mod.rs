use std::error::Error;

use reference::{Reference, ReferencePush};

pub mod evm;
mod executor;
pub mod hash;
pub mod reference;

#[cfg(any(test, feature = "mock"))]
pub mod mock;

#[derive(Debug, PartialEq)]
pub enum RemoteHelperError {
    InvalidHash(String),
    InvalidRpc(String),
    InvalidWalletType(String),
    KeypairPathNotFound,
    RpcNotSet(String),
    UnknownProtocol(String),
}

impl Error for RemoteHelperError {}

impl std::fmt::Display for RemoteHelperError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidHash(hash) => write!(f, "invalid hash: {}", hash),
            Self::InvalidRpc(rpc) => write!(f, "invalid rpc: {:?}", rpc),
            Self::InvalidWalletType(wallet_type) => {
                write!(f, "invalid wallet type: {}", wallet_type)
            }
            Self::KeypairPathNotFound => write!(f, "keypair path not found"),
            Self::RpcNotSet(protocol) => write!(f, "rpc not set for protocol: {}", protocol),
            Self::UnknownProtocol(protocol) => write!(f, "unknown protocol: {}", protocol),
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
