use std::path::PathBuf;
use std::error::Error;

use crate::remote_helper::Wallet;
use crate::config::Config;
#[cfg(test)]
use crate::config::mock::MockConfig;
#[cfg(test)]
use std::collections::HashMap;

static CONFIG_PREFIX: &str = "evm";

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

#[derive(Debug, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    Sepolia,
    Goerli,
    Local,
}

impl Network {
    pub fn from_string(network: String) -> Result<Network, String> {
        match network.as_str() {
            "mainnet" => Ok(Network::Mainnet),
            "sepolia" => Ok(Network::Sepolia),
            "goerli" => Ok(Network::Goerli),
            "local" => Ok(Network::Local),
            _ => Err(format!("Invalid network: {}", network)),
        }
    }
}

pub struct EvmConfig {
    config: Box<dyn Config>,
}

impl EvmConfig {
    pub fn new(config: Box<dyn Config>) -> Self {
        Self { config }
    }

    pub fn get_network(&self) -> Result<Network, Box<dyn Error>> {
        match self.config.read(format!("{}.network", CONFIG_PREFIX).as_str())? {
            Some(network) => Ok(Network::from_string(network)?),
            None => Ok(Network::Mainnet),
        }
    }

    pub fn get_wallet(&self) -> Result<EvmWallet, Box<dyn Error>> {
        match self.config.read(format!("{}.wallet", CONFIG_PREFIX).as_str())? {
            Some(wallet_type) => match wallet_type.as_str() {
                "keypair" => match self.config.read(format!("{}.keypair", CONFIG_PREFIX).as_str())? {
                    Some(keypair_path) => Ok(EvmWallet::Keypair(PathBuf::from(keypair_path))),
                    None => Err("Keypair path not found".into()),
                },
                "environment" => Ok(EvmWallet::Environment),
                "browser" => Ok(EvmWallet::Browser),
                _ => Err("Invalid wallet type".into()),
            },
            None => Ok(EvmWallet::Browser),
        }
    }
}

#[test]
fn test_network() {
    // default network
    let mock_config = MockConfig::new();
    let evm_config = EvmConfig::new(Box::new(mock_config));
    let network = evm_config.get_network().expect("failed to get network");
    assert_eq!(network, Network::Mainnet);

    // mainnet
    let mock_config = MockConfig::new_with_values(HashMap::from([(
        "evm.network".to_string(),
        "mainnet".to_string(),
    )]));
    let evm_config = EvmConfig::new(Box::new(mock_config));
    let network = evm_config.get_network().expect("failed to get network");
    assert_eq!(network, Network::Mainnet);

    // testnet
    let mock_config = MockConfig::new_with_values(HashMap::from([(
        "evm.network".to_string(),
        "sepolia".to_string(),
    )]));
    let evm_config = EvmConfig::new(Box::new(mock_config));
    let network = evm_config.get_network().expect("failed to get network");
    assert_eq!(network, Network::Sepolia);

    // devnet
    let mock_config = MockConfig::new_with_values(HashMap::from([(
        "evm.network".to_string(),
        "goerli".to_string(),
    )]));
    let evm_config = EvmConfig::new(Box::new(mock_config));
    let network = evm_config.get_network().expect("failed to get network");
    assert_eq!(network, Network::Goerli);

    // localnet
    let mock_config = MockConfig::new_with_values(HashMap::from([(
        "evm.network".to_string(),
        "local".to_string(),
    )]));
    let evm_config = EvmConfig::new(Box::new(mock_config));
    let network = evm_config.get_network().expect("failed to get network");
    assert_eq!(network, Network::Local);

    // invalid network
    let mock_config = MockConfig::new_with_values(HashMap::from([(
        "evm.network".to_string(),
        "invalid".to_string(),
    )]));
    let evm_config = EvmConfig::new(Box::new(mock_config));
    evm_config.get_network().expect_err("should fail");
}

#[test]
fn test_wallet() {
    // default
    let mock_config = MockConfig::new();
    let evm_config = EvmConfig::new(Box::new(mock_config));
    let wallet = evm_config
        .get_wallet()
        .expect("failed to get wallet type");
    assert_eq!(wallet, EvmWallet::Browser);

    // browser
    let mock_config = MockConfig::new_with_values(HashMap::from([(
        "evm.wallet".to_string(),
        "browser".to_string(),
    )]));
    let evm_config = EvmConfig::new(Box::new(mock_config));
    let wallet_type = evm_config
        .get_wallet()
        .expect("failed to get wallet type");
    assert_eq!(wallet_type, EvmWallet::Browser);

    // keypair - path provided
    let mock_config = MockConfig::new_with_values(HashMap::from([
        ("evm.wallet".to_string(), "keypair".to_string()),
        ("evm.keypair".to_string(), "/path/to/keypair".to_string()),
    ]));
    let evm_config = EvmConfig::new(Box::new(mock_config));
    let wallet_type = evm_config
        .get_wallet()
        .expect("failed to get wallet type");
    assert_eq!(wallet_type, EvmWallet::Keypair("/path/to/keypair".into()));

    // keypair - path not provided
    let mock_config = MockConfig::new_with_values(HashMap::from([(
        "evm.wallet".to_string(),
        "keypair".to_string(),
    )]));
    let evm_config = EvmConfig::new(Box::new(mock_config));
    evm_config.get_wallet().expect_err("should fail");

    // environment
    let mock_config = MockConfig::new_with_values(HashMap::from([(
        "evm.wallet".to_string(),
        "environment".to_string(),
    )]));
    let evm_config = EvmConfig::new(Box::new(mock_config));
    let wallet_type = evm_config
        .get_wallet()
        .expect("failed to get wallet type");
    assert_eq!(wallet_type, EvmWallet::Environment);

    // invalid wallet type
    let mock_config = MockConfig::new_with_values(HashMap::from([(
        "evm.wallet".to_string(),
        "invalid".to_string(),
    )]));
    let evm_config = EvmConfig::new(Box::new(mock_config));
    evm_config.get_wallet().expect_err("should fail");
}
