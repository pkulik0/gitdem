use std::path::PathBuf;
use std::error::Error;

use crate::remote_helper::Wallet;
use crate::config::Config;
#[cfg(test)]
use crate::config::mock::MockConfig;
#[cfg(test)]
use std::collections::HashMap;

static CONFIG_PREFIX: &str = "solana";

#[derive(Debug, PartialEq, Eq)]
pub enum SolanaWallet {
  Keypair(PathBuf),
  Environment,
  Phantom,
}

impl Wallet for SolanaWallet {
    fn is_extension(&self) -> bool {
        matches!(self, SolanaWallet::Phantom)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    Devnet,
    Testnet,
    Localnet,
}

impl Network {
    pub fn from_string(network: String) -> Result<Network, String> {
        match network.as_str() {
            "mainnet" => Ok(Network::Mainnet),
            "devnet" => Ok(Network::Devnet),
            "testnet" => Ok(Network::Testnet),
            "localnet" => Ok(Network::Localnet),
            _ => Err(format!("Invalid network: {}", network)),
        }
    }
}

pub struct SolanaConfig {
    config: Box<dyn Config>,
}

impl SolanaConfig {
    pub fn new(config: Box<dyn Config>) -> Self {
        Self { config }
    }

    pub fn get_network(&self) -> Result<Network, Box<dyn Error>> {
        match self.config.read(format!("{}.network", CONFIG_PREFIX).as_str())? {
            Some(network) => Ok(Network::from_string(network)?),
            None => Ok(Network::Mainnet),
        }
    }

    pub fn get_wallet(&self) -> Result<SolanaWallet, Box<dyn Error>> {
        match self.config.read(format!("{}.wallet", CONFIG_PREFIX).as_str())? {
            Some(wallet_type) => match wallet_type.as_str() {
                "keypair" => match self.config.read(format!("{}.keypair", CONFIG_PREFIX).as_str())? {
                    Some(keypair_path) => Ok(SolanaWallet::Keypair(PathBuf::from(keypair_path))),
                    None => Err("Keypair path not found".into()),
                },
                "environment" => Ok(SolanaWallet::Environment),
                "phantom" => Ok(SolanaWallet::Phantom),
                _ => Err("Invalid wallet type".into()),
            },
            None => Ok(SolanaWallet::Phantom),
        }
    }
}

#[test]
fn test_network() {
    // default network
    let mock_config = MockConfig::new();
    let solana_config = SolanaConfig::new(Box::new(mock_config));
    let network = solana_config.get_network().expect("failed to get network");
    assert_eq!(network, Network::Mainnet);

    // mainnet
    let mock_config = MockConfig::new_with_values(HashMap::from([(
        "solana.network".to_string(),
        "mainnet".to_string(),
    )]));
    let solana_config = SolanaConfig::new(Box::new(mock_config));
    let network = solana_config.get_network().expect("failed to get network");
    assert_eq!(network, Network::Mainnet);

    // testnet
    let mock_config = MockConfig::new_with_values(HashMap::from([(
        "solana.network".to_string(),
        "testnet".to_string(),
    )]));
    let solana_config = SolanaConfig::new(Box::new(mock_config));
    let network = solana_config.get_network().expect("failed to get network");
    assert_eq!(network, Network::Testnet);

    // devnet
    let mock_config = MockConfig::new_with_values(HashMap::from([(
        "solana.network".to_string(),
        "devnet".to_string(),
    )]));
    let solana_config = SolanaConfig::new(Box::new(mock_config));
    let network = solana_config.get_network().expect("failed to get network");
    assert_eq!(network, Network::Devnet);

    // localnet
    let mock_config = MockConfig::new_with_values(HashMap::from([(
        "solana.network".to_string(),
        "localnet".to_string(),
    )]));
    let solana_config = SolanaConfig::new(Box::new(mock_config));
    let network = solana_config.get_network().expect("failed to get network");
    assert_eq!(network, Network::Localnet);

    // invalid network
    let mock_config = MockConfig::new_with_values(HashMap::from([(
        "solana.network".to_string(),
        "invalid".to_string(),
    )]));
    let solana_config = SolanaConfig::new(Box::new(mock_config));
    solana_config.get_network().expect_err("should fail");
}

#[test]
fn test_wallet() {
    // default
    let mock_config = MockConfig::new();
    let solana_config = SolanaConfig::new(Box::new(mock_config));
    let wallet = solana_config
        .get_wallet()
        .expect("failed to get wallet type");
    assert_eq!(wallet, SolanaWallet::Phantom);

    // phantom
    let mock_config = MockConfig::new_with_values(HashMap::from([(
        "solana.wallet".to_string(),
        "phantom".to_string(),
    )]));
    let solana_config = SolanaConfig::new(Box::new(mock_config));
    let wallet_type = solana_config
        .get_wallet()
        .expect("failed to get wallet type");
    assert_eq!(wallet_type, SolanaWallet::Phantom);

    // keypair - path provided
    let mock_config = MockConfig::new_with_values(HashMap::from([
        ("solana.wallet".to_string(), "keypair".to_string()),
        ("solana.keypair".to_string(), "/path/to/keypair".to_string()),
    ]));
    let solana_config = SolanaConfig::new(Box::new(mock_config));
    let wallet_type = solana_config
        .get_wallet()
        .expect("failed to get wallet type");
    assert_eq!(wallet_type, SolanaWallet::Keypair("/path/to/keypair".into()));

    // keypair - path not provided
    let mock_config = MockConfig::new_with_values(HashMap::from([(
        "solana.wallet".to_string(),
        "keypair".to_string(),
    )]));
    let solana_config = SolanaConfig::new(Box::new(mock_config));
    solana_config.get_wallet().expect_err("should fail");

    // environment
    let mock_config = MockConfig::new_with_values(HashMap::from([(
        "solana.wallet".to_string(),
        "environment".to_string(),
    )]));
    let solana_config = SolanaConfig::new(Box::new(mock_config));
    let wallet_type = solana_config
        .get_wallet()
        .expect("failed to get wallet type");
    assert_eq!(wallet_type, SolanaWallet::Environment);

    // invalid wallet type
    let mock_config = MockConfig::new_with_values(HashMap::from([(
        "solana.wallet".to_string(),
        "invalid".to_string(),
    )]));
    let solana_config = SolanaConfig::new(Box::new(mock_config));
    solana_config.get_wallet().expect_err("should fail");
}
