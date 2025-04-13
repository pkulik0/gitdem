use mockall::predicate::eq;

use crate::core::git::Git;
#[cfg(test)]
use crate::core::git::MockGit;
use crate::core::hash::Hash;
use crate::core::reference::{Push, Reference};
use crate::core::remote_helper::executor::Executor;
#[cfg(test)]
use crate::core::remote_helper::executor::MockExecutor;
use crate::core::remote_helper::{RemoteHelper, RemoteHelperError};
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

    fn fetch(&self, hash: Hash) -> Result<(), RemoteHelperError> {
        let object = self.runtime.block_on(self.executor.fetch(hash))?;
        self.git.save_object(object)?;
        Ok(())
    }

    fn push(&self, pushes: Vec<Push>) -> Result<(), RemoteHelperError> {
        if pushes.is_empty() {
            return Ok(());
        }

        let local_hashes = pushes
            .iter()
            .map(|r| self.git.resolve_reference(&r.local))
            .collect::<Result<Vec<_>, _>>()?;
        let local_names: Vec<String> = pushes.iter().map(|r| r.local.clone()).collect();
        let remote_names: Vec<String> = pushes.into_iter().map(|r| r.remote).collect();

        self.runtime.block_on(async move {
            let remote_hashes = self.executor.resolve_references(remote_names).await?;
            let all_remote_object_hashes = tokio::sync::OnceCell::new();

            let mut references = vec![];
            let mut objects = HashSet::new();
            // There are 3 cases:
            // 1. Local and remote hashes are the same. No need to push anything.
            // 2. Remote hash is empty. The ref doesn't exist so missing objects are calculated
            //    by comparing objects reachable from the local hash with all remote objects.
            // 3. Remote hash is not empty. The ref exists so missing objects are calculated by
            //    getting objects reachable from the local ref but not from the remote one.
            for ((local_hash, remote_hash), local_name) in local_hashes
                .into_iter()
                .zip(remote_hashes.into_iter())
                .zip(local_names.into_iter())
            {
                if local_hash == remote_hash {
                    continue;
                }

                if remote_hash.is_empty() {
                    let all_remote_object_hashes: &HashSet<Hash> =
                        match all_remote_object_hashes.get() {
                            Some(hashes) => hashes,
                            None => {
                                let hashes = self.executor.list_objects().await?;
                                let _ = all_remote_object_hashes.set(hashes.into_iter().collect());
                                all_remote_object_hashes
                                    .get()
                                    .expect("should be set right above")
                            }
                        };

                    let local_ref_hashes: HashSet<Hash> = self
                        .git
                        .list_objects(local_hash.clone())?
                        .into_iter()
                        .collect();

                    let missing_objects = local_ref_hashes
                        .difference(&all_remote_object_hashes)
                        .map(|hash| self.git.get_object(hash.clone()))
                        .collect::<Result<Vec<_>, _>>()?;
                    objects.extend(missing_objects);
                } else {
                    let missing_objects = self
                        .git
                        .list_missing_objects(local_hash.clone(), remote_hash)?
                        .into_iter()
                        .map(|hash| self.git.get_object(hash))
                        .collect::<Result<Vec<_>, _>>()?;
                    objects.extend(missing_objects);
                }

                references.push(Reference::Normal {
                    name: local_name,
                    hash: local_hash,
                });
            }

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
fn test_list() {
    use crate::core::reference::Reference;
    use tokio::runtime::Builder;

    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");
    let mut executor = Box::new(MockExecutor::new());
    executor.expect_list().returning(|| Ok(vec![]));
    let evm = Evm::new(runtime, executor, Box::new(MockGit::new())).expect("should be set");
    let refs = evm.list(false).expect("should be set");
    assert_eq!(refs.len(), 0);

    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");
    let data = b"1234567890";
    let refs = vec![
        Reference::Normal {
            name: "refs/heads/main".to_string(),
            hash: Hash::from_data_sha256(data).expect("should be set"),
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
fn test_fetch() {
    use crate::core::object::{Object, ObjectKind};
    use tokio::runtime::Builder;

    // Case 1: Fetch succeeds
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");
    let mut executor = Box::new(MockExecutor::new());
    let object = Object {
        kind: ObjectKind::Blob,
        data: b"1234567890".to_vec(),
    };
    let object_clone = object.clone();
    executor
        .expect_fetch()
        .returning(move |_| Ok(object_clone.clone()));
    let mut git = Box::new(MockGit::new());
    git.expect_save_object()
        .with(eq(object.clone()))
        .returning(|_| Ok(()));
    let evm = Evm::new(runtime, executor, git).expect("should be set");
    evm.fetch(object.hash(true)).expect("should succeed");

    // Case 2: Fetch fails because the object is missing
    let runtime = Builder::new_current_thread()
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
    let hash = Hash::from_data_sha256(b"1234567890").expect("should be set");
    evm.fetch(hash).expect_err("should fail");

    // Case 3: Fetch fails because the it wasn't saved to .git
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");
    let mut executor = Box::new(MockExecutor::new());
    let object = Object {
        kind: ObjectKind::Blob,
        data: b"abcdef".to_vec(),
    };
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
    evm.fetch(object.hash(true)).expect_err("should fail");
}

#[test]
fn test_push() {
    use crate::core::object::{Object, ObjectKind};
    use crate::core::reference::Push;
    use tokio::runtime::Builder;

    // Case 0: Empty push
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");
    let mut executor = Box::new(MockExecutor::new());
    executor
        .expect_push()
        .returning(|_objects, _references| Ok(()));
    let evm = Evm::new(runtime, executor, Box::new(MockGit::new())).expect("should be set");
    let pushes = vec![];
    evm.push(pushes).expect("should succeed");

    // Case 1: Remote already up to date
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");

    let hash = Hash::from_data_sha256(b"1234567890").expect("should be set");

    let mut executor = Box::new(MockExecutor::new());
    executor
        .expect_push()
        .returning(|_objects, _references| Ok(()));
    let hash_clone = hash.clone();
    executor
        .expect_resolve_references()
        .returning(move |_names| Ok(vec![hash_clone.clone()]));

    let mut git = Box::new(MockGit::new());
    let hash_clone = hash.clone();
    git.expect_resolve_reference()
        .returning(move |_name| Ok(hash_clone.clone()));

    let evm = Evm::new(runtime, executor, git).expect("should be set");
    let pushes = vec![Push {
        local: "refs/heads/main".to_string(),
        remote: "refs/heads/main".to_string(),
        is_force: false,
    }];
    evm.push(pushes).expect("should succeed");

    // Case 2: Remote ref doesn't exist
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");

    let hash = Hash::from_data_sha256(b"1234567890").expect("should be set");
    let object0 = Object {
        kind: ObjectKind::Blob,
        data: b"1234567890".to_vec(),
    };
    let object1 = Object {
        kind: ObjectKind::Blob,
        data: b"abcdef".to_vec(),
    };

    let mut executor = Box::new(MockExecutor::new());
    executor
        .expect_resolve_references()
        .returning(move |_names| Ok(vec![Hash::empty(true)]));
    let object0_hash = object0.hash(true);
    executor
        .expect_list_objects()
        .returning(move || Ok(vec![object0_hash.clone()]));
    executor
        .expect_push()
        .with(
            eq(vec![object1.clone()]),
            eq(vec![Reference::Normal {
                name: "refs/heads/main".to_string(),
                hash: hash.clone(),
            }]),
        )
        .returning(|_objects, _references| Ok(()));

    let mut git = Box::new(MockGit::new());
    let hash_clone = hash.clone();
    git.expect_resolve_reference()
        .returning(move |_name| Ok(hash_clone.clone()));
    let object0_hash = object0.hash(true);
    let object1_hash = object1.hash(true);
    git.expect_list_objects()
        .returning(move |_ref_hash| Ok(vec![object0_hash.clone(), object1_hash.clone()]));
    git.expect_get_object()
        .with(eq(object1.hash(true)))
        .returning(move |_| Ok(object1.clone()));

    let evm = Evm::new(runtime, executor, git).expect("should be set");
    let pushes = vec![Push {
        local: "refs/heads/main".to_string(),
        remote: "refs/heads/main".to_string(),
        is_force: false,
    }];
    evm.push(pushes).expect("should succeed");

    // Case 3: Remote ref exists but is different
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");

    let object0 = Object {
        kind: ObjectKind::Blob,
        data: b"1234567890".to_vec(),
    };
    let hash = Hash::from_data_sha256(b"1234567890").expect("should be set");
    let another_hash = Hash::from_data_sha256(b"abcdef").expect("should be set");

    let mut executor = Box::new(MockExecutor::new());
    let hash_clone = hash.clone();
    executor
        .expect_resolve_references()
        .returning(move |_names| Ok(vec![hash_clone.clone()]));
    executor
        .expect_push()
        .with(
            eq(vec![object0.clone()]),
            eq(vec![Reference::Normal {
                name: "refs/heads/main".to_string(),
                hash: another_hash.clone(),
            }]),
        )
        .returning(|_objects, _references| Ok(()));

    let mut git = Box::new(MockGit::new());
    let another_hash_clone = another_hash.clone();
    git.expect_resolve_reference()
        .returning(move |_name| Ok(another_hash_clone.clone()));
    let object0_hash = object0.hash(true);
    git.expect_list_missing_objects()
        .with(eq(another_hash.clone()), eq(hash.clone()))
        .returning(move |_local_hash, _remote_hash| Ok(vec![object0_hash.clone()]));
    git.expect_get_object()
        .with(eq(object0.hash(true)))
        .returning(move |_object_hash| Ok(object0.clone()));

    let evm = Evm::new(runtime, executor, git).expect("should be set");
    let pushes = vec![Push {
        local: "refs/heads/main".to_string(),
        remote: "refs/heads/main".to_string(),
        is_force: false,
    }];
    evm.push(pushes).expect("should succeed");

    // Failure case 1: Can't resolve local reference
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");

    let mut git = Box::new(MockGit::new());
    git.expect_resolve_reference().returning(|_name| {
        Err(RemoteHelperError::Failure {
            action: "".to_string(),
            details: None,
        })
    });
    let evm = Evm::new(runtime, Box::new(MockExecutor::new()), git).expect("should be set");
    let pushes = vec![Push {
        local: "refs/heads/main".to_string(),
        remote: "refs/heads/main".to_string(),
        is_force: false,
    }];
    evm.push(pushes).expect_err("should fail");

    // Failure case 2: Can't resolve remote reference
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");

    let mut executor = Box::new(MockExecutor::new());
    executor.expect_resolve_references().returning(|_names| {
        Err(RemoteHelperError::Failure {
            action: "".to_string(),
            details: None,
        })
    });
    let mut git = Box::new(MockGit::new());
    git.expect_resolve_reference()
        .returning(|_name| Ok(Hash::from_data_sha256(b"1234567890").expect("should be set")));

    let evm = Evm::new(runtime, executor, git).expect("should be set");
    let pushes = vec![Push {
        local: "refs/heads/main".to_string(),
        remote: "refs/heads/main".to_string(),
        is_force: false,
    }];
    evm.push(pushes).expect_err("should fail");

    // Failure case 3: Remote ref not present, can't list objects
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");

    let mut executor = Box::new(MockExecutor::new());
    executor
        .expect_resolve_references()
        .returning(|_names| Ok(vec![Hash::empty(true)]));
    executor.expect_list_objects().returning(|| {
        Err(RemoteHelperError::Failure {
            action: "".to_string(),
            details: None,
        })
    });

    let mut git = Box::new(MockGit::new());
    git.expect_resolve_reference()
        .returning(|_name| Ok(Hash::from_data_sha256(b"abcdef").expect("should be set")));

    let evm = Evm::new(runtime, executor, git).expect("should be set");
    let pushes = vec![Push {
        local: "refs/heads/main".to_string(),
        remote: "refs/heads/main".to_string(),
        is_force: false,
    }];
    evm.push(pushes).expect_err("should fail");

    // Failure case 4: Remote ref not present, can't list local objects
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");

    let mut executor = Box::new(MockExecutor::new());
    executor
        .expect_resolve_references()
        .returning(|_names| Ok(vec![Hash::empty(true)]));
    executor.expect_list_objects().returning(|| Ok(vec![]));

    let mut git = Box::new(MockGit::new());
    git.expect_resolve_reference()
        .returning(|_name| Ok(Hash::from_data_sha256(b"abcdef").expect("should be set")));
    git.expect_list_objects().returning(|_ref_hash| {
        Err(RemoteHelperError::Failure {
            action: "".to_string(),
            details: None,
        })
    });

    let evm = Evm::new(runtime, executor, git).expect("should be set");
    let pushes = vec![Push {
        local: "refs/heads/main".to_string(),
        remote: "refs/heads/main".to_string(),
        is_force: false,
    }];
    evm.push(pushes).expect_err("should fail");

    // Failure case 5: Remote ref not present, can't get local object
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");

    let mut executor = Box::new(MockExecutor::new());
    executor
        .expect_resolve_references()
        .returning(|_names| Ok(vec![Hash::empty(true)]));
    executor.expect_list_objects().returning(|| Ok(vec![]));

    let mut git = Box::new(MockGit::new());
    git.expect_resolve_reference()
        .returning(|_name| Ok(Hash::from_data_sha256(b"abcdef").expect("should be set")));
    let object = Object {
        kind: ObjectKind::Blob,
        data: b"1234567890".to_vec(),
    };
    let object_hash = object.hash(true);
    git.expect_list_objects()
        .returning(move |_ref_hash| Ok(vec![object_hash.clone()]));
    git.expect_get_object()
        .with(eq(object.hash(true)))
        .returning(|_hash| {
            Err(RemoteHelperError::Failure {
                action: "".to_string(),
                details: None,
            })
        });

    let evm = Evm::new(runtime, executor, git).expect("should be set");
    let pushes = vec![Push {
        local: "refs/heads/main".to_string(),
        remote: "refs/heads/main".to_string(),
        is_force: false,
    }];
    evm.push(pushes).expect_err("should fail");

    // Failure case 6: Present, can't get missing objects
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");

    let mut executor = Box::new(MockExecutor::new());
    executor.expect_resolve_references().returning(|_names| {
        Ok(vec![
            Hash::from_data_sha256(b"1234567890").expect("should be set"),
        ])
    });

    let mut git = Box::new(MockGit::new());
    git.expect_resolve_reference()
        .returning(|_name| Ok(Hash::from_data_sha256(b"abcdef").expect("should be set")));
    git.expect_list_missing_objects()
        .returning(|_local_hash, _remote_hash| {
            Err(RemoteHelperError::Failure {
                action: "".to_string(),
                details: None,
            })
        });

    let evm = Evm::new(runtime, executor, git).expect("should be set");
    let pushes = vec![Push {
        local: "refs/heads/main".to_string(),
        remote: "refs/heads/main".to_string(),
        is_force: false,
    }];
    evm.push(pushes).expect_err("should fail");

    // Failure case 7: Present, can't get local object
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");

    let mut executor = Box::new(MockExecutor::new());
    executor.expect_resolve_references().returning(|_names| {
        Ok(vec![
            Hash::from_data_sha256(b"1234567890").expect("should be set"),
        ])
    });

    let mut git = Box::new(MockGit::new());
    git.expect_resolve_reference()
        .returning(|_name| Ok(Hash::from_data_sha256(b"abcdef").expect("should be set")));
    let object = Object {
        kind: ObjectKind::Blob,
        data: b"1234567890".to_vec(),
    };
    let object_hash = object.hash(true);
    git.expect_list_missing_objects()
        .returning(move |_local_hash, _remote_hash| Ok(vec![object_hash.clone()]));
    git.expect_get_object()
        .with(eq(object.hash(true)))
        .returning(|_hash| {
            Err(RemoteHelperError::Failure {
                action: "".to_string(),
                details: None,
            })
        });

    let evm = Evm::new(runtime, executor, git).expect("should be set");
    let pushes = vec![Push {
        local: "refs/heads/main".to_string(),
        remote: "refs/heads/main".to_string(),
        is_force: false,
    }];
    evm.push(pushes).expect_err("should fail");

    // Failure case 8: Can't push
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");

    let mut executor = Box::new(MockExecutor::new());
    executor.expect_resolve_references().returning(|_names| {
        Ok(vec![
            Hash::from_data_sha256(b"abcdef").expect("should be set"),
        ])
    });
    executor.expect_push().returning(|_objects, _references| {
        Err(RemoteHelperError::Failure {
            action: "".to_string(),
            details: None,
        })
    });

    let mut git = Box::new(MockGit::new());
    git.expect_resolve_reference()
        .returning(|_name| Ok(Hash::from_data_sha256(b"abcdef").expect("should be set")));
    let hash = Hash::from_data_sha256(b"1234567890").expect("should be set");
    let hash_clone = hash.clone();
    git.expect_list_missing_objects()
        .returning(move |_local_hash, _remote_hash| {
            Ok(vec![hash_clone.clone()])
        });
    git.expect_get_object()
        .with(eq(hash))
        .returning(|_hash| {
            Err(RemoteHelperError::Failure {
                action: "".to_string(),
                details: None,
            })
        });
    let evm = Evm::new(runtime, executor, git).expect("should be set");
    let pushes = vec![Push { 
        local: "refs/heads/main".to_string(),
        remote: "refs/heads/main".to_string(),
        is_force: false,
    }];
    evm.push(pushes).expect_err("should fail");
}
