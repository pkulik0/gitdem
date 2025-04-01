use crate::remote_helper::{Reference, RemoteHelper, RemoteHelperError};

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

    fn fetch(&self, reference: &Reference) -> Result<(), RemoteHelperError> {
        Ok(())
    }
}
