use crate::config::Config;
use crate::remote_helper::evm::config::EvmConfig;
use crate::remote_helper::{Reference, ReferencePush, RemoteHelper, RemoteHelperError};
use crate::args::Args;

pub struct Evm {
    args: Args,
    config: EvmConfig,
}

impl Evm {
    pub fn new(args: Args, config: Box<dyn Config>) -> Self {
        Self {
            args,
            config: EvmConfig::new(config),
        }
    }
}

impl RemoteHelper for Evm {
    fn capabilities(&self) -> Vec<&'static str> {
        vec!["*fetch", "*push"]
    }

    fn list(&self, is_for_push: bool) -> Result<Vec<Reference>, RemoteHelperError> {
        Ok(vec![])
    }

    fn fetch(&self, reference: &Reference) -> Result<(), RemoteHelperError> {
        Ok(())
    }

    fn push(&self, reference: &ReferencePush) -> Result<(), RemoteHelperError> {
        Ok(())
    }
}
