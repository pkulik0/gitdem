use GitRepository::{Object as ContractObject, PushData, RefNormal};
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
use crate::core::object::Object;
#[cfg(test)]
use crate::core::object::ObjectKind;
use crate::core::remote_helper::config::Wallet;
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
        wallet_type: Wallet,
        rpc: &str,
        address: [u8; 20],
    ) -> Result<Self, RemoteHelperError> {
        let private_key = match wallet_type {
            #[cfg(test)]
            Wallet::PrivateKey(private_key) => private_key,
            Wallet::Browser => {
                return Err(RemoteHelperError::Failure {
                    action: "creating background executor".to_string(),
                    details: Some("Browser wallet not supported".to_string()),
                });
            }
            Wallet::Keypair(path) => {
                std::fs::read_to_string(path).map_err(|e| RemoteHelperError::Failure {
                    action: "creating background executor".to_string(),
                    details: Some(e.to_string()),
                })?
            }
            Wallet::Environment => {
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
            refs.push(Reference::Normal {
                name: reference.name,
                hash: reference.hash.into(),
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
        objects: Vec<Object>,
        refs: Vec<Reference>,
        is_sha256: bool,
    ) -> Result<(), RemoteHelperError> {
        let mut data: PushData = PushData {
            objects: vec![],
            refs: vec![],
        };

        for object in objects {
            data.objects.push(ContractObject {
                hash: FixedBytes::from_str(object.hash(is_sha256).padded().as_str()).map_err(
                    |e| RemoteHelperError::Failure {
                        action: "converting hash to fixed bytes".to_string(),
                        details: Some(e.to_string()),
                    },
                )?,
                data: Bytes::from(object.serialize()),
            });
        }

        for reference in refs {
            match reference {
                Reference::Normal { name, hash } => {
                    data.refs.push(RefNormal {
                        name: name.clone(),
                        hash: FixedBytes::from_str(hash.padded().as_str()).map_err(|e| {
                            RemoteHelperError::Failure {
                                action: "converting hash to fixed bytes".to_string(),
                                details: Some(e.to_string()),
                            }
                        })?,
                    });
                }
                _ => {
                    return Err(RemoteHelperError::Failure {
                        action: "pushing objects and refs".to_string(),
                        details: Some("Unsupported reference type".to_string()),
                    });
                }
            }
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

    async fn fetch(&self, hash: Hash) -> Result<Object, RemoteHelperError> {
        let hash_bytes = FixedBytes::from_str(hash.padded().as_str()).map_err(|e| {
            RemoteHelperError::Failure {
                action: "converting hash to fixed bytes".to_string(),
                details: Some(e.to_string()),
            }
        })?;
        let object = self
            .contract
            .getObject(hash_bytes)
            .call()
            .await
            .map_err(|e| RemoteHelperError::Failure {
                action: "fetching object".to_string(),
                details: Some(e.to_string()),
            })?;

        let data = object._0;
        let object = Object::deserialize(&data)?;
        Ok(object)
    }

    async fn resolve_references(&self, names: Vec<String>) -> Result<Vec<Hash>, RemoteHelperError> {
        let response = self
            .contract
            .resolveRefs(names.clone())
            .call()
            .await
            .map_err(|e| RemoteHelperError::Failure {
                action: "resolving references".to_string(),
                details: Some(e.to_string()),
            })?;

        let hashes = response._0.into_iter().map(|h| h.into()).collect();
        Ok(hashes)
    }

    async fn list_objects(&self) -> Result<Vec<Hash>, RemoteHelperError> {
        let response = self.contract.getObjectHashes().call().await.map_err(|e| {
            RemoteHelperError::Failure {
                action: "listing objects".to_string(),
                details: Some(e.to_string()),
            }
        })?;

        let hashes = response._0.into_iter().map(|h| h.into()).collect();
        Ok(hashes)
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
        Wallet::PrivateKey(test_signer_pk.to_string()),
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

    let object = Object::new(ObjectKind::Blob, b"test".to_vec()).expect("failed to create object");
    let hash = object.hash(true);
    let objects = vec![object];
    let refs = vec![Reference::Normal {
        name: "refs/heads/main".to_string(),
        hash: hash.clone(),
    }];
    executor
        .push(objects, refs, true)
        .await
        .expect("failed to push");

    let refs = executor.list().await.expect("failed to list references");
    let expected = vec![
        Reference::Normal {
            name: "refs/heads/main".to_string(),
            hash,
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

#[tokio::test]
async fn test_fetch() {
    let executor = setup_test_executor().await;

    let object = Object::new(ObjectKind::Blob, b"test".to_vec()).expect("failed to create object");
    let hash = object.hash(true);
    let objects = vec![object.clone()];
    let refs = vec![Reference::Normal {
        name: "refs/heads/main".to_string(),
        hash: hash.clone(),
    }];
    executor
        .push(objects, refs, true)
        .await
        .expect("failed to push");

    let fetched_object = executor
        .fetch(hash.clone())
        .await
        .expect("failed to fetch object");
    assert_eq!(object, fetched_object);
}

#[tokio::test]
async fn test_get_references() {
    let executor = setup_test_executor().await;

    let object = Object::new(ObjectKind::Blob, b"test".to_vec()).expect("failed to create object");
    let hash = object.hash(true);
    let objects = vec![object];
    let ref_name = "refs/heads/main".to_string();
    let refs = vec![Reference::Normal {
        name: ref_name.clone(),
        hash: hash.clone(),
    }];
    executor
        .push(objects, refs, true)
        .await
        .expect("failed to push");

    let refs = executor
        .resolve_references(vec![ref_name.clone()])
        .await
        .expect("failed to get references");
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0], hash);
}

#[tokio::test]
async fn test_list_objects() {
    let executor = setup_test_executor().await;

    let hashes = executor
        .list_objects()
        .await
        .expect("failed to list objects");
    assert_eq!(hashes.len(), 0);

    let object = Object::new(ObjectKind::Blob, b"test".to_vec()).expect("failed to create object");
    let hash = object.hash(true);
    let objects = vec![object];
    let refs = vec![Reference::Normal {
        name: "refs/heads/main".to_string(),
        hash: hash.clone(),
    }];
    executor
        .push(objects, refs, true)
        .await
        .expect("failed to push");

    let hashes = executor
        .list_objects()
        .await
        .expect("failed to list objects");
    assert_eq!(hashes.len(), 1);
    assert_eq!(hashes[0], hash);
}
