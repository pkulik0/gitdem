use crate::config::Config;
use crate::remote_helper::solana::config::SolanaConfig;
use crate::remote_helper::{Reference, RemoteHelper, RemoteHelperError};
use crate::args::Args;

pub struct Solana {
    args: Args,
    config: SolanaConfig,
}

impl Solana {
    pub fn new(args: Args, config: Box<dyn Config>) -> Self {
        Self {
            args,
            config: SolanaConfig::new(config),
        }
    }
}

impl RemoteHelper for Solana {
    fn capabilities(&self) -> Vec<&'static str> {
        vec!["*fetch", "*push"]
    }

    fn list(&self, is_for_push: bool) -> Result<Vec<Reference>, RemoteHelperError> {
        Ok(vec![])
    }

    fn fetch(&self, reference: &Reference) -> Result<(), RemoteHelperError> {
        Ok(())
    }
}
