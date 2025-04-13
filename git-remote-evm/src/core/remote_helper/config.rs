use std::path::PathBuf;
use std::sync::LazyLock;

use crate::core::kv_source::KeyValueSource;
#[cfg(test)]
use crate::core::kv_source::MockKeyValueSource;
use crate::core::remote_helper::error::RemoteHelperError;
#[cfg(test)]
use mockall::predicate::eq;
use regex::Regex;

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
    kv_source: Box<dyn KeyValueSource>,
}

impl Config {
    fn to_key(&self, key: &str) -> String {
        format!("{}.{}.{}", CONFIG_PREFIX, self.protocol, key)
    }

    pub fn new(protocol: String, kv_source: Box<dyn KeyValueSource>) -> Self {
        Self { protocol, kv_source }
    }

    pub fn get_rpc(&self) -> Result<String, RemoteHelperError> {
        match self.kv_source.read(self.to_key("rpc").as_str()) {
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
        match self.kv_source.read(self.to_key("wallet").as_str()) {
            Some(wallet_type) => match wallet_type.as_str() {
                "keypair" => match self.kv_source.read(self.to_key("keypair").as_str()) {
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
    let mut mock_config = MockKeyValueSource::new();
    mock_config
        .expect_read()
        .with(eq(format!("{}.{}.rpc", CONFIG_PREFIX, protocol)))
        .return_const(None);
    let evm_config = Config::new(protocol.to_string(), Box::new(mock_config));
    let rpc = evm_config.get_rpc().expect("failed to get rpc");
    assert_eq!(rpc, DEFAULT_RPC_ETH);

    let protocol = "arb1";
    let mut mock_config = MockKeyValueSource::new();
    mock_config
        .expect_read()
        .with(eq(format!("{}.{}.rpc", CONFIG_PREFIX, protocol)))
        .return_const(None);
    let evm_config = Config::new(protocol.to_string(), Box::new(mock_config));
    let rpc = evm_config.get_rpc().expect("failed to get rpc");
    assert_eq!(rpc, DEFAULT_RPC_ARB1);

    let protocol = "avax";
    let mut mock_config = MockKeyValueSource::new();
    mock_config
        .expect_read()
        .with(eq(format!("{}.{}.rpc", CONFIG_PREFIX, protocol)))
        .return_const(None);
    let evm_config = Config::new(protocol.to_string(), Box::new(mock_config));
    let rpc = evm_config.get_rpc().expect("failed to get rpc");
    assert_eq!(rpc, DEFAULT_RPC_AVAX);

    let mut mock_config = MockKeyValueSource::new();
    let another_rpc = "https://some-rpc.com";
    mock_config
        .expect_read()
        .with(eq(format!("{}.{}.rpc", CONFIG_PREFIX, protocol)))
        .return_const(Some(another_rpc.to_string()));
    let evm_config = Config::new(protocol.to_string(), Box::new(mock_config));
    let rpc = evm_config.get_rpc().expect("failed to get rpc");
    assert_eq!(rpc, another_rpc);

    let mut mock_config = MockKeyValueSource::new();
    mock_config
        .expect_read()
        .with(eq(format!("{}.{}.rpc", CONFIG_PREFIX, protocol)))
        .return_const(Some("invalid-rpc".to_string()));
    let evm_config = Config::new(protocol.to_string(), Box::new(mock_config));
    evm_config
        .get_rpc()
        .expect_err("should fail because of invalid rpc");

    let protocol = "unknown";
    let mut mock_config = MockKeyValueSource::new();
    mock_config
        .expect_read()
        .with(eq(format!("{}.{}.rpc", CONFIG_PREFIX, protocol)))
        .return_const(None);
    let evm_config = Config::new(protocol.to_string(), Box::new(mock_config));
    evm_config
        .get_rpc()
        .expect_err("should fail because of unknown protocol");
}

#[test]
fn test_wallet() {
    // default
    let protocol = "eth";
    let mut mock_config = MockKeyValueSource::new();
    mock_config
        .expect_read()
        .with(eq(format!("{}.{}.wallet", CONFIG_PREFIX, protocol)))
        .return_const(None);
    let evm_config = Config::new(protocol.to_string(), Box::new(mock_config));
    let wallet = evm_config.get_wallet().expect("failed to get wallet type");
    assert_eq!(wallet, Wallet::Browser);

    // browser
    let mut mock_config = MockKeyValueSource::new();
    mock_config
        .expect_read()
        .with(eq(format!("{}.{}.wallet", CONFIG_PREFIX, protocol)))
        .return_const(Some("browser".to_string()));
    let evm_config = Config::new(protocol.to_string(), Box::new(mock_config));
    let wallet_type = evm_config.get_wallet().expect("failed to get wallet type");
    assert_eq!(wallet_type, Wallet::Browser);

    // keypair - path provided
    let mut mock_config = MockKeyValueSource::new();
    mock_config
        .expect_read()
        .with(eq(format!("{}.{}.wallet", CONFIG_PREFIX, protocol)))
        .return_const(Some("keypair".to_string()));
    let keypair_path = "/path/to/keypair";
    mock_config
        .expect_read()
        .with(eq(format!("{}.{}.keypair", CONFIG_PREFIX, protocol)))
        .return_const(Some(keypair_path.to_string()));
    let evm_config = Config::new(protocol.to_string(), Box::new(mock_config));
    let wallet_type = evm_config.get_wallet().expect("failed to get wallet type");
    assert_eq!(wallet_type, Wallet::Keypair(PathBuf::from(keypair_path)));

    // keypair - path not provided
    let mut mock_config = MockKeyValueSource::new();
    mock_config
        .expect_read()
        .with(eq(format!("{}.{}.wallet", CONFIG_PREFIX, protocol)))
        .return_const(Some("keypair".to_string()));
    mock_config
        .expect_read()
        .with(eq(format!("{}.{}.keypair", CONFIG_PREFIX, protocol)))
        .return_const(None);
    let evm_config = Config::new(protocol.to_string(), Box::new(mock_config));
    evm_config.get_wallet().expect_err("should fail");

    // environment
    let protocol: &str = "arb1";
    let mut mock_config = MockKeyValueSource::new();
    mock_config
        .expect_read()
        .with(eq(format!("{}.{}.wallet", CONFIG_PREFIX, protocol)))
        .return_const(Some("environment".to_string()));
    let evm_config = Config::new(protocol.to_string(), Box::new(mock_config));
    let wallet_type = evm_config.get_wallet().expect("failed to get wallet type");
    assert_eq!(wallet_type, Wallet::Environment);

    // invalid wallet type
    let mut mock_config = MockKeyValueSource::new();
    mock_config
        .expect_read()
        .with(eq(format!("{}.{}.wallet", CONFIG_PREFIX, protocol)))
        .return_const(Some("invalid".to_string()));
    let evm_config = Config::new(protocol.to_string(), Box::new(mock_config));
    evm_config.get_wallet().expect_err("should fail");
}
