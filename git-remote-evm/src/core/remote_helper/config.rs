use std::path::PathBuf;
use std::sync::LazyLock;

use regex::Regex;

use crate::core::kv_source::KeyValueSource;
#[cfg(test)]
use crate::core::kv_source::mock::MockConfig;
use crate::core::remote_helper::error::RemoteHelperError;
#[cfg(test)]
use std::collections::HashMap;

const CONFIG_PREFIX: &str = "evm";
const RPC_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^https?|wss?:\/\/[^\s]+$").expect("failed to create rpc regex"));

#[derive(Debug, PartialEq, Eq)]
pub enum Wallet {
    #[cfg(test)]
    PrivateKey(String),
    Keypair(PathBuf),
    Environment,
    Browser,
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

pub struct Config {
    protocol: String,
    config: Box<dyn KeyValueSource>,
}

impl Config {
    fn to_key(&self, key: &str) -> String {
        format!("{}.{}.{}", CONFIG_PREFIX, self.protocol, key)
    }

    pub fn new(protocol: String, config: Box<dyn KeyValueSource>) -> Self {
        Self { protocol, config }
    }

    pub fn get_rpc(&self) -> Result<String, RemoteHelperError> {
        match self.config.read(self.to_key("rpc").as_str()) {
            Some(rpc) => match RPC_REGEX.is_match(&rpc) {
                true => Ok(rpc),
                false => Err(RemoteHelperError::Invalid {
                    what: "rpc".to_string(),
                    value: rpc,
                }),
            },
            None => match get_default_rpc(&self.protocol) {
                Some(default_rpc) => Ok(default_rpc.to_string()),
                None => Err(RemoteHelperError::Missing {
                    what: "rpc".to_string(),
                }),
            },
        }
    }

    pub fn get_wallet(&self) -> Result<Wallet, RemoteHelperError> {
        match self.config.read(self.to_key("wallet").as_str()) {
            Some(wallet_type) => match wallet_type.as_str() {
                "keypair" => match self.config.read(self.to_key("keypair").as_str()) {
                    Some(keypair_path) => Ok(Wallet::Keypair(PathBuf::from(keypair_path))),
                    None => Err(RemoteHelperError::Missing {
                        what: "keypair path".to_string(),
                    }),
                },
                "environment" => Ok(Wallet::Environment),
                "browser" => Ok(Wallet::Browser),
                _ => Err(RemoteHelperError::Invalid {
                    what: "wallet type".to_string(),
                    value: wallet_type,
                }),
            },
            None => Ok(Wallet::Browser),
        }
    }
}

#[test]
fn test_rpc() {
    let protocol = "eth";
    let evm_config = Config::new(protocol.to_string(), Box::new(MockConfig::new()));
    let rpc = evm_config.get_rpc().expect("failed to get rpc");
    assert_eq!(rpc, DEFAULT_RPC_ETH);

    let protocol = "arb1";
    let evm_config = Config::new(protocol.to_string(), Box::new(MockConfig::new()));
    let rpc = evm_config.get_rpc().expect("failed to get rpc");
    assert_eq!(rpc, DEFAULT_RPC_ARB1);

    let protocol = "avax";
    let evm_config = Config::new(protocol.to_string(), Box::new(MockConfig::new()));
    let rpc = evm_config.get_rpc().expect("failed to get rpc");
    assert_eq!(rpc, DEFAULT_RPC_AVAX);

    let another_rpc = "https://some-rpc.com";
    let mock_config = MockConfig::new_with_values(HashMap::from([(
        evm_config.to_key("rpc"),
        another_rpc.to_string(),
    )]));
    let evm_config = Config::new(protocol.to_string(), Box::new(mock_config));
    let rpc = evm_config.get_rpc().expect("failed to get rpc");
    assert_eq!(rpc, another_rpc);

    let invalid_rpc = "invalid-rpc";
    let mock_config = MockConfig::new_with_values(HashMap::from([(
        evm_config.to_key("rpc"),
        invalid_rpc.to_string(),
    )]));
    let evm_config = Config::new(protocol.to_string(), Box::new(mock_config));
    evm_config
        .get_rpc()
        .expect_err("should fail because of invalid rpc");

    let protocol = "unknown";
    let mock_config = MockConfig::new();
    let evm_config = Config::new(protocol.to_string(), Box::new(mock_config));
    evm_config
        .get_rpc()
        .expect_err("should fail because of unknown protocol");
}

#[test]
fn test_wallet() {
    // default
    let mock_config = MockConfig::new();
    let evm_config = Config::new("eth".to_string(), Box::new(mock_config));
    let wallet = evm_config.get_wallet().expect("failed to get wallet type");
    assert_eq!(wallet, Wallet::Browser);

    // browser
    let mock_config = MockConfig::new_with_values(HashMap::from([(
        evm_config.to_key("wallet"),
        "browser".to_string(),
    )]));
    let evm_config = Config::new("eth".to_string(), Box::new(mock_config));
    let wallet_type = evm_config.get_wallet().expect("failed to get wallet type");
    assert_eq!(wallet_type, Wallet::Browser);

    // keypair - path provided
    let mock_config = MockConfig::new_with_values(HashMap::from([
        (evm_config.to_key("wallet"), "keypair".to_string()),
        (evm_config.to_key("keypair"), "/path/to/keypair".to_string()),
    ]));
    let evm_config = Config::new("eth".to_string(), Box::new(mock_config));
    let wallet_type = evm_config.get_wallet().expect("failed to get wallet type");
    assert_eq!(wallet_type, Wallet::Keypair("/path/to/keypair".into()));

    // keypair - path not provided
    let mock_config = MockConfig::new_with_values(HashMap::from([(
        evm_config.to_key("wallet"),
        "keypair".to_string(),
    )]));
    let evm_config = Config::new("eth".to_string(), Box::new(mock_config));
    evm_config.get_wallet().expect_err("should fail");

    // environment
    let mock_config = MockConfig::new_with_values(HashMap::from([(
        evm_config.to_key("wallet"),
        "environment".to_string(),
    )]));
    let evm_config = Config::new("eth".to_string(), Box::new(mock_config));
    let wallet_type = evm_config.get_wallet().expect("failed to get wallet type");
    assert_eq!(wallet_type, Wallet::Environment);

    // invalid wallet type
    let mock_config = MockConfig::new_with_values(HashMap::from([(
        evm_config.to_key("wallet"),
        "invalid".to_string(),
    )]));
    let evm_config = Config::new("eth".to_string(), Box::new(mock_config));
    evm_config.get_wallet().expect_err("should fail");
}
