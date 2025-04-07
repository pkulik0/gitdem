use alloy::network::EthereumWallet;
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
