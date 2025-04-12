use crate::core::git::Git;
use crate::core::hash::Hash;
use crate::core::object::Object;
use crate::core::remote_helper::error::RemoteHelperError;
use std::cell::RefCell;

pub struct Mock {
    objects: RefCell<Vec<Object>>, // Because the trait doesn't have a &mut self for save_object
    missing_objects: Vec<Hash>,
}

impl Mock {
    pub fn new(objects: Vec<Object>, missing_objects: Vec<Hash>) -> Self {
        Self {
            objects: RefCell::new(objects),
            missing_objects,
        }
    }
}

impl Git for Mock {
    fn get_object(&self, hash: Hash) -> Result<Object, RemoteHelperError> {
        let object = self
            .objects
            .borrow()
            .iter()
            .find(|object| object.hash == hash)
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

    fn get_missing_objects(
        &self,
        local: Hash,
        remote: Hash,
    ) -> Result<Vec<Hash>, RemoteHelperError> {
        Ok(self.missing_objects.clone())
    }
}
