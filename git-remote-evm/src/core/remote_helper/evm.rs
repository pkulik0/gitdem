use crate::args::Args;
use crate::core::config::Config;
use crate::core::reference::{Reference, ReferencePush};
use crate::core::remote_helper::config::EvmConfig;
use crate::core::remote_helper::{RemoteHelper, RemoteHelperError};

pub struct Evm {
    args: Args,
    config: EvmConfig,
}

impl Evm {
    pub fn new(args: Args, config: Box<dyn Config>) -> Self {
        let protocol = args.protocol().to_string();
        Self {
            args,
            config: EvmConfig::new(protocol, config),
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
