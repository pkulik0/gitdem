use crate::core::git::Git;
#[cfg(test)]
use crate::core::git::MockGit;
use crate::core::hash::Hash;
use crate::core::reference::{Fetch, Push, Reference};
use crate::core::remote_helper::executor::Executor;
#[cfg(test)]
use crate::core::remote_helper::executor::MockExecutor;
use crate::core::remote_helper::{RemoteHelper, RemoteHelperError};
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
        Ok(())
    }

    fn push(&self, pushes: Vec<Push>) -> Result<(), RemoteHelperError> {
        if pushes.is_empty() {
            return Ok(());
        }

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
            let remote_object_hashes = self.executor.list_objects().await?;

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
                debug!("no changes to push");
                return Ok(());
            }
            debug!(
                "pushing objects: {:?} and references: {:?}",
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
fn test_fetch() {
    use crate::core::object::{Object, ObjectKind};
    use tokio::runtime::Builder;

    // Case 1: Fetch succeeds (one object)
    let runtime = Builder::new_current_thread()
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

    // Case 2: Fetch succeeds (more objects)
    let object_blob = Object::new(ObjectKind::Blob, b"1234567890".to_vec(), true)
        .expect("failed to create object");
    let hash_bytes = hex::decode(object_blob.get_hash().to_string()).expect("should succeed");
    let mut tree_data = b"100644 file\0".to_vec();
    tree_data.extend(hash_bytes);
    let object_tree =
        Object::new(ObjectKind::Tree, tree_data, true).expect("failed to create object");

    let runtime = Builder::new_current_thread()
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

    // Case 3: Fetch fails because the object is missing
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
    let hash = Hash::from_data(b"1234567890", true).expect("should be set");
    evm.fetch(vec![Fetch {
        hash,
        name: "refs/heads/main".to_string(),
    }])
    .expect_err("should fail");

    // Case 4: Fetch fails because saving failed
    let runtime = Builder::new_current_thread()
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

    let hash = Hash::from_data(b"1234567890", true).expect("should be set");

    let mut executor = Box::new(MockExecutor::new());
    executor
        .expect_list_objects()
        .returning(|| Ok(vec![Hash::from_data(b"abc", true).expect("should succeed")]));
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

    let hash = Hash::from_data(b"1234567890", true).expect("should be set");
    let object0 = Object::new(ObjectKind::Blob, b"1234567890".to_vec(), true)
        .expect("failed to create object");
    let object1 =
        Object::new(ObjectKind::Blob, b"abcdef".to_vec(), true).expect("failed to create object");

    let mut executor = Box::new(MockExecutor::new());
    executor
        .expect_resolve_references()
        .returning(move |_names| Ok(vec![Hash::empty(true)]));
    let object0_hash = object0.get_hash().clone();
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
    let object0_hash = object0.get_hash().clone();
    let object1_hash = object1.get_hash().clone();
    git.expect_list_objects()
        .returning(move |_ref_hash| Ok(vec![object0_hash.clone(), object1_hash.clone()]));
    git.expect_get_object()
        .with(eq(object1.get_hash().clone()))
        .returning(move |_| Ok(object1.clone()));
    git.expect_is_sha256().returning(|| Ok(true));

    let evm = Evm::new(runtime, executor, git).expect("should be set");
    let pushes = vec![Push {
        local: "refs/heads/main".to_string(),
        remote: "refs/heads/main".to_string(),
        is_force: false,
    }];
    evm.push(pushes).expect("should succeed");

    // // Case 3: Remote ref exists but is different
    // let runtime = Builder::new_current_thread()
    //     .enable_all()
    //     .build()
    //     .expect("failed to build runtime");

    // let object0 = Object::new(ObjectKind::Blob, b"1234567890".to_vec(), true)
    //     .expect("failed to create object");
    // let hash = Hash::from_data(b"1234567890", true).expect("should be set");
    // let another_hash = Hash::from_data(b"abcdef", true).expect("should be set");

    // let mut executor = Box::new(MockExecutor::new());
    // let hash_clone = hash.clone();
    // executor
    //     .expect_resolve_references()
    //     .returning(move |_names| Ok(vec![hash_clone.clone()]));
    // executor
    //     .expect_list_objects()
    //     .returning(|| Ok(vec![object0.hash(true)]));
    // executor
    //     .expect_push()
    //     .with(
    //         eq(vec![object0.clone()]),
    //         eq(vec![Reference::Normal {
    //             name: "refs/heads/main".to_string(),
    //             hash: another_hash.clone(),
    //         }]),
    //         eq(true),
    //     )
    //     .returning(|_objects, _references, _is_sha256| Ok(()));

    // let mut git = Box::new(MockGit::new());
    // let another_hash_clone = another_hash.clone();
    // git.expect_resolve_reference()
    //     .returning(move |_name| Ok(another_hash_clone.clone()));
    // let object0_hash = object0.hash(true);
    // git.expect_list_missing_objects()
    //     .with(eq(another_hash.clone()), eq(hash.clone()))
    //     .returning(move |_local_hash, _remote_hash| Ok(vec![object0_hash.clone()]));
    // git.expect_get_object()
    //     .with(eq(object0.hash(true)))
    //     .returning(move |_object_hash| Ok(object0.clone()));
    // git.expect_is_sha256().returning(|| Ok(true));

    // let evm = Evm::new(runtime, executor, git).expect("should be set");
    // let pushes = vec![Push {
    //     local: "refs/heads/main".to_string(),
    //     remote: "refs/heads/main".to_string(),
    //     is_force: false,
    // }];
    // evm.push(pushes).expect("should succeed");

    // // Failure case 1: Can't resolve local reference
    // let runtime = Builder::new_current_thread()
    //     .enable_all()
    //     .build()
    //     .expect("failed to build runtime");

    // let mut git = Box::new(MockGit::new());
    // git.expect_resolve_reference().returning(|_name| {
    //     Err(RemoteHelperError::Failure {
    //         action: "".to_string(),
    //         details: None,
    //     })
    // });
    // let evm = Evm::new(runtime, Box::new(MockExecutor::new()), git).expect("should be set");
    // let pushes = vec![Push {
    //     local: "refs/heads/main".to_string(),
    //     remote: "refs/heads/main".to_string(),
    //     is_force: false,
    // }];
    // evm.push(pushes).expect_err("should fail");

    // // Failure case 2: Can't resolve remote reference
    // let runtime = Builder::new_current_thread()
    //     .enable_all()
    //     .build()
    //     .expect("failed to build runtime");

    // let mut executor = Box::new(MockExecutor::new());
    // executor.expect_resolve_references().returning(|_names| {
    //     Err(RemoteHelperError::Failure {
    //         action: "".to_string(),
    //         details: None,
    //     })
    // });
    // let mut git = Box::new(MockGit::new());
    // git.expect_resolve_reference()
    //     .returning(|_name| Ok(Hash::from_data(b"1234567890", true).expect("should be set")));

    // let evm = Evm::new(runtime, executor, git).expect("should be set");
    // let pushes = vec![Push {
    //     local: "refs/heads/main".to_string(),
    //     remote: "refs/heads/main".to_string(),
    //     is_force: false,
    // }];
    // evm.push(pushes).expect_err("should fail");

    // // Failure case 3: Remote ref not present, can't list objects
    // let runtime = Builder::new_current_thread()
    //     .enable_all()
    //     .build()
    //     .expect("failed to build runtime");

    // let mut executor = Box::new(MockExecutor::new());
    // executor
    //     .expect_resolve_references()
    //     .returning(|_names| Ok(vec![Hash::empty(true)]));
    // executor.expect_list_objects().returning(|| {
    //     Err(RemoteHelperError::Failure {
    //         action: "".to_string(),
    //         details: None,
    //     })
    // });

    // let mut git = Box::new(MockGit::new());
    // git.expect_resolve_reference()
    //     .returning(|_name| Ok(Hash::from_data(b"abcdef", true).expect("should be set")));

    // let evm = Evm::new(runtime, executor, git).expect("should be set");
    // let pushes = vec![Push {
    //     local: "refs/heads/main".to_string(),
    //     remote: "refs/heads/main".to_string(),
    //     is_force: false,
    // }];
    // evm.push(pushes).expect_err("should fail");

    // // Failure case 4: Remote ref not present, can't list local objects
    // let runtime = Builder::new_current_thread()
    //     .enable_all()
    //     .build()
    //     .expect("failed to build runtime");

    // let mut executor = Box::new(MockExecutor::new());
    // executor
    //     .expect_resolve_references()
    //     .returning(|_names| Ok(vec![Hash::empty(true)]));
    // executor.expect_list_objects().returning(|| Ok(vec![]));

    // let mut git = Box::new(MockGit::new());
    // git.expect_resolve_reference()
    //     .returning(|_name| Ok(Hash::from_data(b"abcdef", true).expect("should be set")));
    // git.expect_list_objects().returning(|_ref_hash| {
    //     Err(RemoteHelperError::Failure {
    //         action: "".to_string(),
    //         details: None,
    //     })
    // });

    // let evm = Evm::new(runtime, executor, git).expect("should be set");
    // let pushes = vec![Push {
    //     local: "refs/heads/main".to_string(),
    //     remote: "refs/heads/main".to_string(),
    //     is_force: false,
    // }];
    // evm.push(pushes).expect_err("should fail");

    // // Failure case 5: Remote ref not present, can't get local object
    // let runtime = Builder::new_current_thread()
    //     .enable_all()
    //     .build()
    //     .expect("failed to build runtime");

    // let mut executor = Box::new(MockExecutor::new());
    // executor
    //     .expect_resolve_references()
    //     .returning(|_names| Ok(vec![Hash::empty(true)]));
    // executor.expect_list_objects().returning(|| Ok(vec![]));

    // let mut git = Box::new(MockGit::new());
    // git.expect_resolve_reference()
    //     .returning(|_name| Ok(Hash::from_data(b"abcdef", true).expect("should be set")));
    // let object = Object::new(ObjectKind::Blob, b"1234567890".to_vec(), true)
    //     .expect("failed to create object");
    // let object_hash = object.hash(true);
    // git.expect_list_objects()
    //     .returning(move |_ref_hash| Ok(vec![object_hash.clone()]));
    // git.expect_get_object()
    //     .with(eq(object.hash(true)))
    //     .returning(|_hash| {
    //         Err(RemoteHelperError::Failure {
    //             action: "".to_string(),
    //             details: None,
    //         })
    //     });

    // let evm = Evm::new(runtime, executor, git).expect("should be set");
    // let pushes = vec![Push {
    //     local: "refs/heads/main".to_string(),
    //     remote: "refs/heads/main".to_string(),
    //     is_force: false,
    // }];
    // evm.push(pushes).expect_err("should fail");

    // // Failure case 6: Present, can't get missing objects
    // let runtime = Builder::new_current_thread()
    //     .enable_all()
    //     .build()
    //     .expect("failed to build runtime");

    // let mut executor = Box::new(MockExecutor::new());
    // executor.expect_resolve_references().returning(|_names| {
    //     Ok(vec![
    //         Hash::from_data(b"1234567890", true).expect("should be set"),
    //     ])
    // });

    // let mut git = Box::new(MockGit::new());
    // git.expect_resolve_reference()
    //     .returning(|_name| Ok(Hash::from_data(b"abcdef", true).expect("should be set")));
    // git.expect_list_missing_objects()
    //     .returning(|_local_hash, _remote_hash| {
    //         Err(RemoteHelperError::Failure {
    //             action: "".to_string(),
    //             details: None,
    //         })
    //     });

    // let evm = Evm::new(runtime, executor, git).expect("should be set");
    // let pushes = vec![Push {
    //     local: "refs/heads/main".to_string(),
    //     remote: "refs/heads/main".to_string(),
    //     is_force: false,
    // }];
    // evm.push(pushes).expect_err("should fail");

    // // Failure case 7: Present, can't get local object
    // let runtime = Builder::new_current_thread()
    //     .enable_all()
    //     .build()
    //     .expect("failed to build runtime");

    // let mut executor = Box::new(MockExecutor::new());
    // executor.expect_resolve_references().returning(|_names| {
    //     Ok(vec![
    //         Hash::from_data(b"1234567890", true).expect("should be set"),
    //     ])
    // });

    // let mut git = Box::new(MockGit::new());
    // git.expect_resolve_reference()
    //     .returning(|_name| Ok(Hash::from_data(b"abcdef", true).expect("should be set")));
    // let object = Object::new(ObjectKind::Blob, b"1234567890".to_vec(), true)
    //     .expect("failed to create object");
    // let object_hash = object.hash(true);
    // git.expect_list_missing_objects()
    //     .returning(move |_local_hash, _remote_hash| Ok(vec![object_hash.clone()]));
    // git.expect_get_object()
    //     .with(eq(object.hash(true)))
    //     .returning(|_hash| {
    //         Err(RemoteHelperError::Failure {
    //             action: "".to_string(),
    //             details: None,
    //         })
    //     });

    // let evm = Evm::new(runtime, executor, git).expect("should be set");
    // let pushes = vec![Push {
    //     local: "refs/heads/main".to_string(),
    //     remote: "refs/heads/main".to_string(),
    //     is_force: false,
    // }];
    // evm.push(pushes).expect_err("should fail");

    // // Failure case 8: Can't push
    // let runtime = Builder::new_current_thread()
    //     .enable_all()
    //     .build()
    //     .expect("failed to build runtime");

    // let mut executor = Box::new(MockExecutor::new());
    // executor.expect_resolve_references().returning(|_names| {
    //     Ok(vec![
    //         Hash::from_data(b"abcdef", true).expect("should be set"),
    //     ])
    // });
    // executor
    //     .expect_push()
    //     .returning(|_objects, _references, _is_sha256| {
    //         Err(RemoteHelperError::Failure {
    //             action: "".to_string(),
    //             details: None,
    //         })
    //     });

    // let mut git = Box::new(MockGit::new());
    // git.expect_resolve_reference()
    //     .returning(|_name| Ok(Hash::from_data(b"ebebeb", true).expect("should be set")));
    // let hash = Hash::from_data(b"1234567890", true).expect("should be set");
    // let hash_clone = hash.clone();
    // git.expect_list_missing_objects()
    //     .returning(move |_local_hash, _remote_hash| Ok(vec![hash_clone.clone()]));
    // git.expect_get_object().with(eq(hash)).returning(|_hash| {
    //     Err(RemoteHelperError::Failure {
    //         action: "".to_string(),
    //         details: None,
    //     })
    // });
    // let evm = Evm::new(runtime, executor, git).expect("should be set");
    // let pushes = vec![Push {
    //     local: "refs/heads/main".to_string(),
    //     remote: "refs/heads/main".to_string(),
    //     is_force: false,
    // }];
    // evm.push(pushes).expect_err("should fail");
}
