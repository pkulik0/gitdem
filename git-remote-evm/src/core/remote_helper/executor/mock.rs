use super::Executor;
use crate::core::{
    hash::Hash,
    object::{Object, ObjectKind},
    reference::Reference,
    remote_helper::error::RemoteHelperError,
};
use async_trait::async_trait;

pub struct Mock;

impl Mock {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Executor for Mock {
    async fn list(&self) -> Result<Vec<Reference>, RemoteHelperError> {
        Ok(vec![])
    }

    async fn push(
        &self,
        objects: Vec<Object>,
        references: Vec<Reference>,
    ) -> Result<(), RemoteHelperError> {
        Ok(())
    }

    async fn fetch(&self, _hash: Hash) -> Result<Object, RemoteHelperError> {
        Ok(Object::new(ObjectKind::Blob, vec![]))
    }

    async fn resolve_references(
        &self,
        _names: Vec<String>,
    ) -> Result<Vec<Hash>, RemoteHelperError> {
        Ok(vec![])
    }

    async fn list_objects(&self) -> Result<Vec<Hash>, RemoteHelperError> {
        Ok(vec![])
    }
}
