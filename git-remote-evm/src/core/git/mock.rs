use crate::core::git::Git;
use crate::core::hash::Hash;
use crate::core::object::Object;
use crate::core::remote_helper::error::RemoteHelperError;
use std::cell::RefCell;
use std::collections::HashMap;

pub struct Mock {
    objects: RefCell<Vec<Object>>, // Because the trait doesn't have a &mut self for save_object
    missing_objects: Vec<Hash>,
    references: HashMap<String, Hash>,
}

impl Mock {
    pub fn new(objects: Vec<Object>, missing_objects: Vec<Hash>, references: HashMap<String, Hash>) -> Self {
        Self {
            objects: RefCell::new(objects),
            missing_objects,
            references,
        }
    }
}

impl Git for Mock {
    fn resolve_reference(&self, name: &str) -> Result<Hash, RemoteHelperError> {
        let hash = self.references.get(name).ok_or(RemoteHelperError::Missing {
            what: format!("reference {} not found", name),
        })?;
        Ok(hash.clone())
    }

    fn get_object(&self, hash: Hash) -> Result<Object, RemoteHelperError> {
        let object = self
            .objects
            .borrow()
            .iter()
            .find(|object| object.hash(true) == hash)
            .ok_or(RemoteHelperError::Missing {
                what: "object not found".to_string(),
            })?
            .clone();
        Ok(object)
    }

    fn save_object(&self, object: Object) -> Result<(), RemoteHelperError> {
        self.objects.borrow_mut().push(object);
        Ok(())
    }

    fn list_missing_objects(
        &self,
        local: Hash,
        remote: Hash,
    ) -> Result<Vec<Hash>, RemoteHelperError> {
        Ok(self.missing_objects.clone())
    }
}
