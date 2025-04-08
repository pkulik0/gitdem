use alloy::network::EthereumWallet;
#[cfg(test)]
use alloy::primitives::Address;
use alloy::providers::fillers::{
    BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller, WalletFiller,
};
use alloy::providers::{Identity, ProviderBuilder, RootProvider};
use alloy::signers::local::PrivateKeySigner;
use alloy::sol;
use async_trait::async_trait;
use std::str::FromStr;

use super::Executor;
use crate::core::hash::Hash;
use crate::core::remote_helper::config::EvmWallet;
use crate::core::{
    reference::{Keys, Reference},
    remote_helper::error::RemoteHelperError,
};

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    GitRepository,
    "../on-chain/artifacts/contracts/GitRepository.sol/GitRepository.json"
);

type Provider = FillProvider<
    JoinFill<
        JoinFill<
            Identity,
            JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
        >,
        WalletFiller<EthereumWallet>,
    >,
    RootProvider,
>;

pub struct Background {
    contract: GitRepository::GitRepositoryInstance<(), Provider>,
}

impl Background {
    pub async fn new(
        wallet_type: EvmWallet,
        rpc: &str,
        address: [u8; 20],
    ) -> Result<Self, RemoteHelperError> {
        let private_key = match wallet_type {
            #[cfg(test)]
            EvmWallet::PrivateKey(private_key) => private_key,
            EvmWallet::Browser => {
                return Err(RemoteHelperError::Failure {
                    action: "creating background executor".to_string(),
                    details: Some("Browser wallet not supported".to_string()),
                });
            }
            EvmWallet::Keypair(path) => {
                std::fs::read_to_string(path).map_err(|e| RemoteHelperError::Failure {
                    action: "creating background executor".to_string(),
                    details: Some(e.to_string()),
                })?
            }
            EvmWallet::Environment => {
                std::env::var("GITDEM_PRIVATE_KEY").map_err(|e| RemoteHelperError::Failure {
                    action: "creating background executor".to_string(),
                    details: Some(e.to_string()),
                })?
            }
        };

        let signer =
            private_key
                .parse::<PrivateKeySigner>()
                .map_err(|e| RemoteHelperError::Failure {
                    action: "parsing private key".to_string(),
                    details: Some(e.to_string()),
                })?;
        let wallet = EthereumWallet::from(signer);

        let provider = ProviderBuilder::new()
            .wallet(wallet)
            .connect(rpc)
            .await
            .map_err(|e| RemoteHelperError::Failure {
                action: "creating background executor".to_string(),
                details: Some(e.to_string()),
            })?;

        Ok(Self {
            contract: GitRepository::new(address.into(), provider),
        })
    }
}

#[async_trait]
impl Executor for Background {
    async fn list(&self) -> Result<Vec<Reference>, RemoteHelperError> {
        let response =
            self.contract
                .listRefs()
                .call()
                .await
                .map_err(|e| RemoteHelperError::Failure {
                    action: "listing references".to_string(),
                    details: Some(e.to_string()),
                })?;

        let normal = response._0.normal;
        let symbolic = response._0.symbolic;
        let kv = response._0.kv;

        let mut refs = vec![];

        for reference in normal {
            let hash = Hash::from_str(reference.hash.to_string().as_str())?;
            refs.push(Reference::Normal {
                name: reference.name,
                hash,
            });
        }
        for reference in symbolic {
            refs.push(Reference::Symbolic {
                name: reference.name,
                target: reference.target,
            });
        }
        for reference in kv {
            let key = Keys::from_str(reference.key.as_str())?;
            refs.push(Reference::KeyValue {
                key: key,
                value: reference.value,
            });
        }

        Ok(refs)
    }
}

#[cfg(test)]
async fn deploy_contract(deployer_pk: &str, rpc: &str) -> Address {
    let signer = deployer_pk
        .parse::<PrivateKeySigner>()
        .expect("failed to parse deployer private key");
    let wallet = EthereumWallet::from(signer);

    let provider = ProviderBuilder::new()
        .wallet(wallet)
        .on_http(rpc.parse().expect("failed to parse rpc"));

    let contract = GitRepository::deploy(provider, true)
        .await
        .expect("failed to deploy contract");
    contract.address().to_owned()
}

#[cfg(test)]
const TEST_SIGNER0_PK: &str = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
#[cfg(test)]
const TEST_RPC: &str = "http://localhost:8545";

#[tokio::test]
async fn test_list() {
    let address = deploy_contract(TEST_SIGNER0_PK, TEST_RPC).await;

    let executor = Background::new(
        EvmWallet::PrivateKey(TEST_SIGNER0_PK.to_string()),
        TEST_RPC,
        address.into(),
    )
    .await
    .expect("failed to create executor");

    let refs = executor.list().await.expect("failed to list references");
    let expected = vec![
        Reference::Symbolic {
            name: "HEAD".to_string(),
            target: "refs/heads/main".to_string(),
        },
        Reference::KeyValue {
            key: Keys::ObjectFormat,
            value: "sha256".to_string(),
        },
    ];
    assert_eq!(refs, expected);
}
