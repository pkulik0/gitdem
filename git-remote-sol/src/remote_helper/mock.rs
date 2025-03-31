use crate::remote_helper::{Reference, RemoteHelper, RemoteHelperError};

pub struct Mock {
    refs: Vec<Reference>,
}

impl Mock {
    pub fn new() -> Self {
        Self {
            refs: vec![],
        }
    }

    pub fn new_with_refs(refs: Vec<Reference>) -> Self {
        Self { refs }
    }
}

impl RemoteHelper for Mock {
    fn capabilities(&self) -> Vec<&'static str> {
        vec!["*fetch", "*push"]
    }

    fn list(&self) -> Result<Vec<Reference>, RemoteHelperError> {
        Ok(self.refs.clone())
    }
}
