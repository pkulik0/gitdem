use crate::remote_helper::RemoteHelper;

pub struct Solana {}

impl Solana {
    pub fn new() -> Self {
        Self {}
    }
}

impl RemoteHelper for Solana {
    fn capabilities(&self) -> Vec<&'static str> {
        vec!["fetch", "push"]
    }
}
