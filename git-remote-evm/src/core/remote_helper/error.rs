use std::error::Error;

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
