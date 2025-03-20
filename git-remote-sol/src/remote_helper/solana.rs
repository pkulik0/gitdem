use crate::remote_helper::RemoteHelper;
use crate::config::Config;
use std::error::Error;
use std::path::PathBuf;

static CONFIG_PREFIX: &str = "solana";


#[derive(Debug, PartialEq, Eq)]
pub enum Wallet {
  Keypair(PathBuf),
  Environment,
  Phantom,
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

    pub fn get_wallet(&self) -> Result<Wallet, Box<dyn Error>> {
        match self.config.read(format!("{}.wallet", CONFIG_PREFIX).as_str())? {
            Some(wallet_type) => match wallet_type.as_str() {
                "keypair" => match self.config.read(format!("{}.keypair", CONFIG_PREFIX).as_str())? {
                    Some(keypair_path) => Ok(Wallet::Keypair(PathBuf::from(keypair_path))),
                    None => Err("Keypair path not found".into()),
                },
                "environment" => Ok(Wallet::Environment),
                "phantom" => Ok(Wallet::Phantom),
                _ => Err("Invalid wallet type".into()),
            },
            None => Ok(Wallet::Phantom),
        }
    }
}

pub struct Solana {
    config: SolanaConfig,
}

impl Solana {
    pub fn new(config: Box<dyn Config>) -> Self {
        Self { config: SolanaConfig::new(config) }
    }
}

impl RemoteHelper for Solana {
    fn capabilities(&self) -> Vec<&'static str> {
        vec!["fetch", "push"]
    }
}
