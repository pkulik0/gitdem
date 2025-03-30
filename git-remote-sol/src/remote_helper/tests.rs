use crate::{config::mock::MockConfig, remote_helper::solana::config::{Network, SolanaConfig, SolanaWallet}};
use std::collections::HashMap;

#[test]
fn solana_config_network() {
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
fn solana_config_wallet() {
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
