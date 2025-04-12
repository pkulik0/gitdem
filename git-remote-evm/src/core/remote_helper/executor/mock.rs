use super::Executor;
use crate::core::{
    hash::Hash, object::Object, reference::Reference, remote_helper::error::RemoteHelperError,
};
use async_trait::async_trait;
use std::sync::RwLock;

pub struct Mock {
    objects: RwLock<Vec<Object>>,
    refs: RwLock<Vec<Reference>>,
}

impl Mock {
    pub fn new(objects: Vec<Object>, refs: Vec<Reference>) -> Self {
        Self {
            objects: RwLock::new(objects),
            refs: RwLock::new(refs),
        }
    }
}

#[async_trait]
impl Executor for Mock {
    async fn list(&self) -> Result<Vec<Reference>, RemoteHelperError> {
        Ok(self.refs.read().expect("failed to read refs").clone())
    }

    async fn push(
        &self,
        objects: Vec<Object>,
        references: Vec<Reference>,
    ) -> Result<(), RemoteHelperError> {
        self.objects
            .write()
            .expect("failed to write objects")
            .extend(objects);
        self.refs
            .write()
            .expect("failed to write refs")
            .extend(references);
        Ok(())
    }

    async fn fetch(&self, hash: Hash) -> Result<Object, RemoteHelperError> {
        let objects = self.objects.read().expect("failed to read objects");
        let object = objects
            .iter()
            .find(|object| object.hash(true) == hash)
            .ok_or(RemoteHelperError::Missing {
                what: "object".to_string(),
            })?;
        Ok(object.clone())
    }

    async fn resolve_references(
        &self,
        _names: Vec<String>,
    ) -> Result<Vec<Hash>, RemoteHelperError> {
        let refs = self.refs.read().expect("failed to read refs");
        let hashes = refs
            .iter()
            .filter_map(|reference| {
                if let Reference::Normal { hash, .. } = reference {
                    Some(hash.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        Ok(hashes)
    }

    async fn list_objects(&self) -> Result<Vec<Hash>, RemoteHelperError> {
        let objects = self.objects.read().expect("failed to read objects");
        let hashes = objects
            .iter()
            .map(|object| object.hash(true))
            .collect::<Vec<_>>();
        Ok(hashes)
    }
}
