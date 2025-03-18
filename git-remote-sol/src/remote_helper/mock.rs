use crate::remote_helper::RemoteHelper;

pub struct Mock {}

impl Mock {
    pub fn new() -> Self {
        Self {}
    }
}

impl RemoteHelper for Mock {
    fn capabilities(&self) -> Vec<&'static str> {
        vec!["fetch", "push"]
    }
}
