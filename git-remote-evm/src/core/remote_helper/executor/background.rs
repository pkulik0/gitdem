use GitRepository::{Object, PushData, RefNormal};
use alloy::network::EthereumWallet;
use alloy::primitives::{Bytes, FixedBytes};
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
use crate::core::object::Object as GitObject;
use crate::core::reference::ReferencePush;
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
            // remove the 0x prefix
            let raw_hash = &reference.hash.to_string()[2..];
            let hash = Hash::from_str(raw_hash)?;
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

    async fn push(
        &self,
        objects: Vec<GitObject>,
        refs: Vec<ReferencePush>,
    ) -> Result<(), RemoteHelperError> {
        let mut data: PushData = PushData {
            objects: vec![],
            refs: vec![],
        };

        for object in objects {
            data.objects.push(Object {
                hash: FixedBytes::from_str(object.hash.to_string().as_str()).map_err(|e| {
                    RemoteHelperError::Failure {
                        action: "pushing objects and refs".to_string(),
                        details: Some(e.to_string()),
                    }
                })?,
                data: Bytes::from(object.data),
            });
        }

        for reference in refs {
            data.refs.push(RefNormal {
                name: reference.src,
                hash: FixedBytes::from_str(reference.dest.as_str()).map_err(|e| {
                    RemoteHelperError::Failure {
                        action: "pushing objects and refs".to_string(),
                        details: Some(e.to_string()),
                    }
                })?,
            });
        }

        let pending_tx = self
            .contract
            .pushObjectsAndRefs(data)
            .send()
            .await
            .map_err(|e| RemoteHelperError::Failure {
                action: "pushing objects and refs".to_string(),
                details: Some(e.to_string()),
            })?;
        pending_tx
            .with_required_confirmations(1)
            .get_receipt()
            .await
            .map_err(|e| RemoteHelperError::Failure {
                action: "pushing objects and refs".to_string(),
                details: Some(e.to_string()),
            })?;

        Ok(())
    }
}

#[cfg(test)]
async fn setup_test_executor() -> Background {
    let test_signer_pk = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let test_rpc = "http://localhost:8545";

    let signer = test_signer_pk
        .parse::<PrivateKeySigner>()
        .expect("failed to parse deployer private key");
    let wallet = EthereumWallet::from(signer);

    let provider = ProviderBuilder::new()
        .wallet(wallet)
        .on_http(test_rpc.parse().expect("failed to parse rpc"));

    let contract = GitRepository::deploy(provider, true)
        .await
        .expect("failed to deploy contract");

    let executor = Background::new(
        EvmWallet::PrivateKey(test_signer_pk.to_string()),
        test_rpc,
        contract.address().to_owned().into(),
    )
    .await
    .expect("failed to create executor");

    executor
}

#[tokio::test]
async fn test_list() {
    let executor = setup_test_executor().await;

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

#[tokio::test]
async fn test_push() {
    let executor = setup_test_executor().await;

    let hash = Hash::from_str("9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08")
        .expect("failed to parse hash");
    let objects = vec![GitObject {
        hash: hash.clone(),
        data: b"test".to_vec(),
    }];
    let refs = vec![ReferencePush {
        src: "refs/heads/main".to_string(),
        dest: hash.to_string(),
        is_force: false,
    }];
    executor.push(objects, refs).await.expect("failed to push");

    let refs = executor.list().await.expect("failed to list references");
    let expected = vec![
        Reference::Normal {
            name: "refs/heads/main".to_string(),
            hash: hash,
        },
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
