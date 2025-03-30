use crate::remote_helper::{Reference, RemoteHelper};
use crate::config::Config;
use std::error::Error;
use std::path::PathBuf;

static CONFIG_PREFIX: &str = "solana";

pub trait Wallet {
    fn is_extension(&self) -> bool;
}


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

    fn list(&self) -> Result<Vec<Reference>, Box<dyn Error>> {
        Ok(vec![])
    }   
}
