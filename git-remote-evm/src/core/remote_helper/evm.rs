use crate::core::git::Git;
#[cfg(test)]
use crate::core::git::MockGit;
use crate::core::hash::Hash;
#[cfg(test)]
use crate::core::object::{Object, ObjectKind};
use crate::core::reference::{Fetch, Push, Reference};
use crate::core::remote_helper::executor::Executor;
#[cfg(test)]
use crate::core::remote_helper::executor::MockExecutor;
use crate::core::remote_helper::{RemoteHelper, RemoteHelperError};
use crate::print_user;
use log::debug;
#[cfg(test)]
use mockall::predicate::eq;
use std::collections::HashSet;

pub struct Evm {
    runtime: tokio::runtime::Runtime,
    executor: Box<dyn Executor>,
    git: Box<dyn Git>,
}

impl Evm {
    pub fn new(
        runtime: tokio::runtime::Runtime,
        executor: Box<dyn Executor>,
        git: Box<dyn Git>,
    ) -> Result<Self, RemoteHelperError> {
        Ok(Self {
            runtime,
            executor,
            git,
        })
    }
}

impl RemoteHelper for Evm {
    fn capabilities(&self) -> Vec<&'static str> {
        vec!["*fetch", "*push"]
    }

    fn list(&self, _is_for_push: bool) -> Result<Vec<Reference>, RemoteHelperError> {
        self.runtime.block_on(self.executor.list())
    }

    fn fetch(&self, fetches: Vec<Fetch>) -> Result<(), RemoteHelperError> {
        print_user!("fetching {} references", fetches.len());
        let mut to_fetch: Vec<Hash> = fetches.into_iter().map(|f| f.hash).collect();
        let mut processed = HashSet::new();
        while let Some(hash) = to_fetch.pop() {
            if !processed.insert(hash.clone()) {
                continue;
            }
            let object = self.runtime.block_on(self.executor.fetch(hash))?;
            to_fetch.extend(object.get_related().iter().cloned());
            self.git.save_object(object)?;
        }
        print_user!("got {} new objects", processed.len());
        Ok(())
    }

    fn push(&self, pushes: Vec<Push>) -> Result<(), RemoteHelperError> {
        if pushes.is_empty() {
            print_user!("nothing to push");
            return Ok(());
        }

        print_user!("calculating required updates");

        let local_ref_hashes = pushes
            .iter()
            .map(|push| self.git.resolve_reference(&push.local))
            .collect::<Result<Vec<_>, _>>()?;

        self.runtime.block_on(async move {
            let remote_ref_names: Vec<String> =
                pushes.into_iter().map(|push| push.remote).collect();
            let remote_ref_hashes = self
                .executor
                .resolve_references(remote_ref_names.clone())
                .await?;
            let remote_object_hashes = self.executor.list_all_objects().await?;

            let mut references = Vec::new();
            let mut objects = HashSet::new();
            for ((local_hash, remote_hash), remote_ref_name) in local_ref_hashes
                .into_iter()
                .zip(remote_ref_hashes.into_iter())
                .zip(remote_ref_names.into_iter())
            {
                if local_hash == remote_hash {
                    debug!("remote ref {} is up to date", remote_ref_name);
                    continue;
                }

                references.push(Reference::Normal {
                    name: remote_ref_name.clone(),
                    hash: local_hash.clone(),
                });
                objects.extend(
                    self.git
                        .list_objects(local_hash.clone())?
                        .into_iter()
                        .filter(|hash| !remote_object_hashes.contains(hash))
                        .map(|hash| self.git.get_object(hash.clone()))
                        .collect::<Result<Vec<_>, _>>()?,
                );
            }

            if objects.is_empty() && references.is_empty() {
                print_user!("no changes to push");
                return Ok(());
            }
            print_user!(
                "pushing {} object{} and {} reference{}",
                objects.len(),
                if objects.len() == 1 { "" } else { "s" },
                references.len(),
                if references.len() == 1 { "" } else { "s" },
            );
            debug!(
                "objects: {:?}, references: {:?}",
                objects, references
            );
            self.executor
                .push(objects.into_iter().collect(), references)
                .await
        })
    }
}

#[test]
fn test_capabilities() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");
    let evm = Evm::new(
        runtime,
        Box::new(MockExecutor::new()),
        Box::new(MockGit::new()),
    )
    .expect("should be set");
    assert_eq!(evm.capabilities(), vec!["*fetch", "*push"]);
}

#[test]
fn test_list_empty() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");
    let mut executor = Box::new(MockExecutor::new());
    executor.expect_list().returning(|| Ok(vec![]));
    let evm = Evm::new(runtime, executor, Box::new(MockGit::new())).expect("should be set");
    let refs = evm.list(false).expect("should be set");
    assert_eq!(refs.len(), 0);
}

#[test]
fn test_list_normal() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");
    let data = b"1234567890";
    let refs = vec![
        Reference::Normal {
            name: "refs/heads/main".to_string(),
            hash: Hash::from_data(data, true).expect("should be set"),
        },
        Reference::Symbolic {
            name: "HEAD".to_string(),
            target: "refs/heads/main".to_string(),
        },
    ];
    let mut executor = Box::new(MockExecutor::new());
    let refs_clone = refs.clone();
    executor
        .expect_list()
        .returning(move || Ok(refs_clone.clone()));
    let evm = Evm::new(runtime, executor, Box::new(MockGit::new())).expect("should be set");
    let returned_refs = evm.list(true).expect("should be set");
    assert_eq!(refs, returned_refs);
}

#[test]
fn test_list_failure() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");

    let mut executor = Box::new(MockExecutor::new());
    executor.expect_list().returning(|| {
        Err(RemoteHelperError::Failure {
            action: "list".to_string(),
            details: Some("object".to_string()),
        })
    });
    let evm = Evm::new(runtime, executor, Box::new(MockGit::new())).expect("should be set");
    evm.list(true).expect_err("should fail");
}

#[test]
fn test_fetch_one() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");
    let mut executor = Box::new(MockExecutor::new());
    let object = Object::new(ObjectKind::Blob, b"1234567890".to_vec(), true)
        .expect("failed to create object");
    let object_clone = object.clone();
    executor
        .expect_fetch()
        .returning(move |_| Ok(object_clone.clone()));
    let mut git = Box::new(MockGit::new());
    git.expect_save_object()
        .with(eq(object.clone()))
        .returning(|_| Ok(()));
    let evm = Evm::new(runtime, executor, git).expect("should be set");
    evm.fetch(vec![Fetch {
        hash: object.get_hash().clone(),
        name: "refs/heads/main".to_string(),
    }])
    .expect("should succeed");
}

#[test]
fn test_fetch_multiple() {
    let object_blob = Object::new(ObjectKind::Blob, b"1234567890".to_vec(), true)
        .expect("failed to create object");
    let hash_bytes = hex::decode(object_blob.get_hash().to_string()).expect("should succeed");
    let mut tree_data = b"100644 file\0".to_vec();
    tree_data.extend(hash_bytes);
    let object_tree =
        Object::new(ObjectKind::Tree, tree_data, true).expect("failed to create object");

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");
    let mut executor = Box::new(MockExecutor::new());
    let object_blob_clone = object_blob.clone();
    let object_tree_clone = object_tree.clone();
    executor
        .expect_fetch()
        .with(eq(object_blob_clone.get_hash().clone()))
        .returning(move |_| Ok(object_blob_clone.clone()));
    executor
        .expect_fetch()
        .with(eq(object_tree_clone.get_hash().clone()))
        .returning(move |_| Ok(object_tree_clone.clone()));

    let mut git = Box::new(MockGit::new());
    let object_tree_clone = object_tree.clone();
    git.expect_save_object()
        .with(eq(object_tree_clone.clone()))
        .returning(|_| Ok(()));
    let object_blob_clone = object_blob.clone();
    git.expect_save_object()
        .with(eq(object_blob_clone.clone()))
        .returning(|_| Ok(()));

    let evm = Evm::new(runtime, executor, git).expect("should be set");
    evm.fetch(vec![Fetch {
        hash: object_tree.get_hash().clone(),
        name: "refs/heads/main".to_string(),
    }])
    .expect("should succeed");
}

#[test]
fn test_fetch_missing() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");
    let mut executor = Box::new(MockExecutor::new());
    executor.expect_fetch().returning(|_| {
        Err(RemoteHelperError::Missing {
            what: "object".to_string(),
        })
    });
    let evm = Evm::new(runtime, executor, Box::new(MockGit::new())).expect("should be set");
    let hash = Hash::from_data(b"1234567890", true).expect("should be set");
    evm.fetch(vec![Fetch {
        hash,
        name: "refs/heads/main".to_string(),
    }])
    .expect_err("should fail");
}

#[test]
fn test_fetch_failure() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");

    let mut executor = Box::new(MockExecutor::new());
    executor.expect_fetch().returning(|_| {
        Err(RemoteHelperError::Failure {
            action: "fetch".to_string(),
            details: Some("object".to_string()),
        })
    });

    let evm = Evm::new(runtime, executor, Box::new(MockGit::new())).expect("should be set");
    evm.fetch(vec![Fetch {
        hash: Hash::from_data(b"1234567890", true).expect("should be set"),
        name: "refs/heads/main".to_string(),
    }])
    .expect_err("should fail");
}

#[test]
fn test_fetch_save_failure() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");
    let mut executor = Box::new(MockExecutor::new());
    let object =
        Object::new(ObjectKind::Blob, b"abcdef".to_vec(), true).expect("failed to create object");
    let object_clone = object.clone();
    executor
        .expect_fetch()
        .returning(move |_| Ok(object_clone.clone()));
    let mut git = Box::new(MockGit::new());
    git.expect_save_object()
        .with(eq(object.clone()))
        .returning(|_| {
            Err(RemoteHelperError::Failure {
                action: "save".to_string(),
                details: Some("object".to_string()),
            })
        });
    let evm = Evm::new(runtime, executor, git).expect("should be set");
    evm.fetch(vec![Fetch {
        hash: object.get_hash().clone(),
        name: "refs/heads/main".to_string(),
    }])
    .expect_err("should fail");
}

#[test]
fn test_push_empty() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");
    let evm = Evm::new(
        runtime,
        Box::new(MockExecutor::new()),
        Box::new(MockGit::new()),
    )
    .expect("should be set");
    evm.push(vec![]).expect("should succeed");
}

#[test]
fn test_push_up_to_date() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");
    let hash = Hash::from_data(b"1234567890", true).expect("should be set");

    let mut executor = Box::new(MockExecutor::new());
    let hash_clone = hash.clone();
    executor
        .expect_resolve_references()
        .returning(move |_| Ok(vec![hash_clone.clone()]));
    executor.expect_list_all_objects().returning(|| Ok(vec![]));

    let mut git = Box::new(MockGit::new());
    git.expect_resolve_reference()
        .with(eq("refs/heads/main".to_string()))
        .returning(move |_| Ok(hash.clone()));

    let evm = Evm::new(runtime, executor, git).expect("should be set");
    evm.push(vec![Push {
        local: "refs/heads/main".to_string(),
        remote: "refs/heads/main".to_string(),
        is_force: false,
    }])
    .expect("should succeed");
}

#[test]
fn test_push_no_new_objects() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");

    let object_hash = Hash::from_data(b"object_data", true).expect("should be set");
    let new_ref_hash = Hash::from_data(b"ref_two", true).expect("should be set");

    let mut executor = Box::new(MockExecutor::new());
    executor.expect_resolve_references().returning(move |_| {
        Ok(vec![
            Hash::from_data(b"ref_one", true).expect("should be set"),
        ])
    });
    let object_hash_clone = object_hash.clone();
    executor
        .expect_list_all_objects()
        .returning(move || Ok(vec![object_hash_clone.clone()]));
    executor
        .expect_push()
        .with(
            eq(vec![]),
            eq(vec![Reference::Normal {
                name: "refs/heads/main".to_string(),
                hash: new_ref_hash.clone(),
            }]),
        )
        .returning(move |_, _| Ok(()));

    let mut git = Box::new(MockGit::new());
    git.expect_resolve_reference()
        .returning(move |_| Ok(new_ref_hash.clone()));
    git.expect_list_objects()
        .returning(move |_| Ok(vec![object_hash.clone()]));

    let evm = Evm::new(runtime, executor, git).expect("should be set");
    evm.push(vec![Push {
        local: "refs/heads/main".to_string(),
        remote: "refs/heads/main".to_string(),
        is_force: false,
    }])
    .expect("should succeed");
}

#[test]
fn test_push_new_object() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");

    let object =
        Object::new(ObjectKind::Blob, b"object_data".to_vec(), true).expect("should be set");
    let new_ref_hash = Hash::from_data(b"ref_two", true).expect("should be set");

    let mut executor = Box::new(MockExecutor::new());
    executor.expect_resolve_references().returning(move |_| {
        Ok(vec![
            Hash::from_data(b"ref_one", true).expect("should be set"),
        ])
    });
    executor
        .expect_list_all_objects()
        .returning(move || Ok(vec![]));
    let object_clone = object.clone();
    executor
        .expect_push()
        .with(
            eq(vec![object_clone]),
            eq(vec![Reference::Normal {
                name: "refs/heads/main".to_string(),
                hash: new_ref_hash.clone(),
            }]),
        )
        .returning(move |_, _| Ok(()));

    let mut git = Box::new(MockGit::new());
    git.expect_resolve_reference()
        .returning(move |_| Ok(new_ref_hash.clone()));
    let object_hash = object.get_hash().clone();
    git.expect_list_objects()
        .returning(move |_| Ok(vec![object_hash.clone()]));
    let object_hash = object.get_hash().clone();
    git.expect_get_object()
        .with(eq(object_hash.clone()))
        .returning(move |_| Ok(object.clone()));

    let evm = Evm::new(runtime, executor, git).expect("should be set");
    evm.push(vec![Push {
        local: "refs/heads/main".to_string(),
        remote: "refs/heads/main".to_string(),
        is_force: false,
    }])
    .expect("should succeed");
}

#[test]
fn test_push_resolve_local_reference_failure() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");

    let mut git = Box::new(MockGit::new());
    git.expect_resolve_reference().returning(move |_| {
        Err(RemoteHelperError::Failure {
            action: "resolve references".to_string(),
            details: Some("object".to_string()),
        })
    });

    let evm = Evm::new(runtime, Box::new(MockExecutor::new()), git).expect("should be set");
    evm.push(vec![Push {
        local: "refs/heads/main".to_string(),
        remote: "refs/heads/main".to_string(),
        is_force: false,
    }])
    .expect_err("should fail");
}

#[test]
fn test_push_resolve_remote_reference_failure() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");

    let mut executor = Box::new(MockExecutor::new());
    executor.expect_resolve_references().returning(|_| {
        Err(RemoteHelperError::Failure {
            action: "resolve references".to_string(),
            details: Some("object".to_string()),
        })
    });

    let mut git = Box::new(MockGit::new());
    git.expect_resolve_reference()
        .returning(|_| Hash::from_data(b"ref_one", true));

    let evm = Evm::new(runtime, executor, git).expect("should be set");
    evm.push(vec![Push {
        local: "refs/heads/main".to_string(),
        remote: "refs/heads/main".to_string(),
        is_force: false,
    }])
    .expect_err("should fail");
}

#[test]
fn test_push_list_remote_objects_failure() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");

    let mut executor = Box::new(MockExecutor::new());
    executor.expect_resolve_references().returning(|_| {
        Ok(vec![
            Hash::from_data(b"ref_one", true).expect("should be set"),
        ])
    });
    executor.expect_list_all_objects().returning(|| {
        Err(RemoteHelperError::Failure {
            action: "list objects".to_string(),
            details: Some("object".to_string()),
        })
    });

    let mut git = Box::new(MockGit::new());
    git.expect_resolve_reference()
        .returning(|_| Hash::from_data(b"ref_two", true));

    let evm = Evm::new(runtime, executor, git).expect("should be set");
    evm.push(vec![Push {
        local: "refs/heads/main".to_string(),
        remote: "refs/heads/main".to_string(),
        is_force: false,
    }])
    .expect_err("should fail");
}

#[test]
fn test_push_list_local_objects_failure() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");

    let mut executor = Box::new(MockExecutor::new());
    executor.expect_list_all_objects().returning(|| Ok(vec![]));
    executor.expect_resolve_references().returning(|_| {
        Ok(vec![
            Hash::from_data(b"ref_one", true).expect("should be set"),
        ])
    });

    let mut git = Box::new(MockGit::new());
    git.expect_resolve_reference()
        .returning(|_| Hash::from_data(b"ref_two", true));
    git.expect_list_objects().returning(|_| {
        Err(RemoteHelperError::Failure {
            action: "list objects".to_string(),
            details: Some("object".to_string()),
        })
    });

    let evm = Evm::new(runtime, executor, git).expect("should be set");
    evm.push(vec![Push {
        local: "refs/heads/main".to_string(),
        remote: "refs/heads/main".to_string(),
        is_force: false,
    }])
    .expect_err("should fail");
}

#[test]
fn test_push_get_object_failure() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");

    let mut executor = Box::new(MockExecutor::new());
    executor.expect_list_all_objects().returning(|| Ok(vec![]));
    executor.expect_resolve_references().returning(|_| {
        Ok(vec![
            Hash::from_data(b"ref_one", true).expect("should be set"),
        ])
    });

    let mut git = Box::new(MockGit::new());
    git.expect_resolve_reference()
        .returning(|_| Hash::from_data(b"ref_two", true));
    git.expect_list_objects().returning(|_| {
        Ok(vec![
            Hash::from_data(b"object_hash", true).expect("should be set"),
        ])
    });
    git.expect_get_object().returning(|_| {
        Err(RemoteHelperError::Failure {
            action: "get object".to_string(),
            details: Some("object".to_string()),
        })
    });

    let evm = Evm::new(runtime, executor, git).expect("should be set");
    evm.push(vec![Push {
        local: "refs/heads/main".to_string(),
        remote: "refs/heads/main".to_string(),
        is_force: false,
    }])
    .expect_err("should fail");
}

#[test]
fn test_push_failure() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");

    let mut executor = Box::new(MockExecutor::new());
    executor.expect_list_all_objects().returning(|| Ok(vec![]));
    executor.expect_resolve_references().returning(|_| {
        Ok(vec![
            Hash::from_data(b"ref_one", true).expect("should be set"),
        ])
    });
    executor.expect_push().returning(|_, _| {
        Err(RemoteHelperError::Failure {
            action: "push".to_string(),
            details: Some("object".to_string()),
        })
    });

    let object =
        Object::new(ObjectKind::Blob, b"object_data".to_vec(), true).expect("should be set");

    let mut git = Box::new(MockGit::new());
    git.expect_resolve_reference()
        .returning(|_| Hash::from_data(b"ref_two", true));
    let object_hash = object.get_hash().clone();
    git.expect_list_objects()
        .returning(move |_| Ok(vec![object_hash.clone()]));
    git.expect_get_object()
        .returning(move |_| Ok(object.clone()));

    let evm = Evm::new(runtime, executor, git).expect("should be set");
    evm.push(vec![Push {
        local: "refs/heads/main".to_string(),
        remote: "refs/heads/main".to_string(),
        is_force: false,
    }])
    .expect_err("should fail");
}
