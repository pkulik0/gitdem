use std::error::Error;
use std::path::PathBuf;
use std::sync::LazyLock;

use regex::Regex;

use crate::config::Config;
#[cfg(test)]
use crate::config::mock::MockConfig;
use crate::remote_helper::RemoteHelperError;
use crate::remote_helper::Wallet;
#[cfg(test)]
use std::collections::HashMap;

const CONFIG_PREFIX: &str = "evm";
const RPC_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^https?|wss?:\/\/[^\s]+$").expect("failed to create rpc regex"));

#[derive(Debug, PartialEq, Eq)]
pub enum EvmWallet {
    Keypair(PathBuf),
    Environment,
    Browser,
}

impl Wallet for EvmWallet {
    fn is_extension(&self) -> bool {
        matches!(self, EvmWallet::Browser)
    }
}

const DEFAULT_RPC_ETH: &str = "https://eth.llamarpc.com";
const DEFAULT_RPC_ARB1: &str = "wss://arbitrum-one-rpc.publicnode.com";
const DEFAULT_RPC_AVAX: &str = "wss://avalanche-c-chain-rpc.publicnode.com";

fn get_default_rpc(protocol: &str) -> Option<&str> {
    match protocol {
        "eth" => Some(DEFAULT_RPC_ETH),
        "arb1" => Some(DEFAULT_RPC_ARB1),
        "avax" => Some(DEFAULT_RPC_AVAX),
        _ => None,
    }
}

pub struct EvmConfig {
    protocol: String,
    config: Box<dyn Config>,
}

impl EvmConfig {
    fn to_key(&self, key: &str) -> String {
        format!("{}.{}.{}", CONFIG_PREFIX, self.protocol, key)
    }

    pub fn new(protocol: String, config: Box<dyn Config>) -> Self {
        Self { protocol, config }
    }

    pub fn get_rpc(&self) -> Result<String, RemoteHelperError> {
        match self.config.read(self.to_key("rpc").as_str()) {
            Some(rpc) => match RPC_REGEX.is_match(&rpc) {
                true => Ok(rpc),
                false => Err(RemoteHelperError::InvalidRpc(rpc)),
            },
            None => match get_default_rpc(&self.protocol) {
                Some(default_rpc) => Ok(default_rpc.to_string()),
                None => Err(RemoteHelperError::RpcNotSet(self.protocol.clone())),
            },
        }
    }

    pub fn get_wallet(&self) -> Result<EvmWallet, RemoteHelperError> {
        match self.config.read(self.to_key("wallet").as_str()) {
            Some(wallet_type) => match wallet_type.as_str() {
                "keypair" => match self.config.read(self.to_key("keypair").as_str()) {
                    Some(keypair_path) => Ok(EvmWallet::Keypair(PathBuf::from(keypair_path))),
                    None => Err(RemoteHelperError::KeypairPathNotFound),
                },
                "environment" => Ok(EvmWallet::Environment),
                "browser" => Ok(EvmWallet::Browser),
                _ => Err(RemoteHelperError::InvalidWalletType(wallet_type)),
            },
            None => Ok(EvmWallet::Browser),
        }
    }
}

#[test]
fn test_rpc() {
    let protocol = "eth";
    let evm_config = EvmConfig::new(protocol.to_string(), Box::new(MockConfig::new()));
    let rpc = evm_config.get_rpc().expect("failed to get rpc");
    assert_eq!(rpc, DEFAULT_RPC_ETH);

    let protocol = "arb1";
    let evm_config = EvmConfig::new(protocol.to_string(), Box::new(MockConfig::new()));
    let rpc = evm_config.get_rpc().expect("failed to get rpc");
    assert_eq!(rpc, DEFAULT_RPC_ARB1);
    
    let protocol = "avax";
    let evm_config = EvmConfig::new(protocol.to_string(), Box::new(MockConfig::new()));
    let rpc = evm_config.get_rpc().expect("failed to get rpc");
    assert_eq!(rpc, DEFAULT_RPC_AVAX);

    let another_rpc = "https://some-rpc.com";
    let mock_config = MockConfig::new_with_values(HashMap::from([(
        evm_config.to_key("rpc"),
        another_rpc.to_string(),
    )]));
    let evm_config = EvmConfig::new(protocol.to_string(), Box::new(mock_config));
    let rpc = evm_config.get_rpc().expect("failed to get rpc");
    assert_eq!(rpc, another_rpc);

    let invalid_rpc = "invalid-rpc";
    let mock_config = MockConfig::new_with_values(HashMap::from([(
        evm_config.to_key("rpc"),
        invalid_rpc.to_string(),
    )]));
    let evm_config = EvmConfig::new(protocol.to_string(), Box::new(mock_config));
    evm_config.get_rpc().expect_err("should fail because of invalid rpc");

    let protocol = "unknown";
    let mock_config = MockConfig::new();
    let evm_config = EvmConfig::new(protocol.to_string(), Box::new(mock_config));
    evm_config.get_rpc().expect_err("should fail because of unknown protocol");
}

#[test]
fn test_wallet() {
    // default
    let mock_config = MockConfig::new();
    let evm_config = EvmConfig::new("eth".to_string(), Box::new(mock_config));
    let wallet = evm_config.get_wallet().expect("failed to get wallet type");
    assert_eq!(wallet, EvmWallet::Browser);

    // browser
    let mock_config = MockConfig::new_with_values(HashMap::from([(
        evm_config.to_key("wallet"),
        "browser".to_string(),
    )]));
    let evm_config = EvmConfig::new("eth".to_string(), Box::new(mock_config));
    let wallet_type = evm_config.get_wallet().expect("failed to get wallet type");
    assert_eq!(wallet_type, EvmWallet::Browser);

    // keypair - path provided
    let mock_config = MockConfig::new_with_values(HashMap::from([
        (evm_config.to_key("wallet"), "keypair".to_string()),
        (evm_config.to_key("keypair"), "/path/to/keypair".to_string()),
    ]));
    let evm_config = EvmConfig::new("eth".to_string(), Box::new(mock_config));
    let wallet_type = evm_config.get_wallet().expect("failed to get wallet type");
    assert_eq!(wallet_type, EvmWallet::Keypair("/path/to/keypair".into()));

    // keypair - path not provided
    let mock_config = MockConfig::new_with_values(HashMap::from([(
        evm_config.to_key("wallet"),
        "keypair".to_string(),
    )]));
    let evm_config = EvmConfig::new("eth".to_string(), Box::new(mock_config));
    evm_config.get_wallet().expect_err("should fail");

    // environment
    let mock_config = MockConfig::new_with_values(HashMap::from([(
        evm_config.to_key("wallet"),
        "environment".to_string(),
    )]));
    let evm_config = EvmConfig::new("eth".to_string(), Box::new(mock_config));
    let wallet_type = evm_config.get_wallet().expect("failed to get wallet type");
    assert_eq!(wallet_type, EvmWallet::Environment);

    // invalid wallet type
    let mock_config = MockConfig::new_with_values(HashMap::from([(
        evm_config.to_key("wallet"),
        "invalid".to_string(),
    )]));
    let evm_config = EvmConfig::new("eth".to_string(), Box::new(mock_config));
    evm_config.get_wallet().expect_err("should fail");
}
