use super::remote_helper::error::RemoteHelperError;
use crate::core::hash::Hash;
use crate::core::object::Object;

// #[cfg(feature = "mock")]
pub mod mock;
pub mod system;

pub trait Git {
    fn resolve_reference(&self, name: &str) -> Result<Hash, RemoteHelperError>;
    fn get_object(&self, hash: Hash) -> Result<Object, RemoteHelperError>;
    fn save_object(&self, object: Object) -> Result<(), RemoteHelperError>;
    fn list_missing_objects(
        &self,
        local: Hash,
        remote: Hash,
    ) -> Result<Vec<Hash>, RemoteHelperError>;
}
