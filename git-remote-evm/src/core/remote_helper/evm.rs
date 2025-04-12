use crate::core::git::Git;
use crate::core::hash::Hash;
use crate::core::reference::{Push, Reference};
use crate::core::remote_helper::executor::Executor;
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
