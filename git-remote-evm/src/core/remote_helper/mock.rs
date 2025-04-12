use crate::core::hash::Hash;
use crate::core::reference::{Reference, Push};
use crate::core::remote_helper::{RemoteHelper, RemoteHelperError};

pub struct Mock {
    refs: Vec<Reference>,
}

impl Mock {
    pub fn new(refs: Vec<Reference>) -> Self {
        Self { refs }
    }
}

impl RemoteHelper for Mock {
    fn capabilities(&self) -> Vec<&'static str> {
        vec!["*fetch", "*push"]
    }

    fn list(&self, _is_for_push: bool) -> Result<Vec<Reference>, RemoteHelperError> {
        Ok(self.refs.clone())
    }

    fn fetch(&self, hash: Hash) -> Result<(), RemoteHelperError> {
        Ok(())
    }

    fn push(&self, _pushes: Vec<Push>) -> Result<(), RemoteHelperError> {
        Ok(())
    }
}
