use crate::args::Args;
use crate::core::config::Config;
use crate::core::git::Git;
use crate::core::hash::Hash;
use crate::core::reference::{Reference, ReferencePush};
use crate::core::remote_helper::config::EvmConfig;
use crate::core::remote_helper::executor::{Executor, create_executor};
use crate::core::remote_helper::{RemoteHelper, RemoteHelperError};

pub struct Evm {
    args: Args,
    config: EvmConfig,
    executor: Box<dyn Executor>,
    runtime: tokio::runtime::Runtime,
    git: Box<dyn Git>,
}

impl Evm {
    pub fn new(
        args: Args,
        config: Box<dyn Config>,
        git: Box<dyn Git>,
    ) -> Result<Self, RemoteHelperError> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| RemoteHelperError::Failure {
                action: "creating runtime".to_string(),
                details: Some(e.to_string()),
            })?;

        let protocol = args.protocol().to_string();
        let config = EvmConfig::new(protocol, config);
        let executor =
            runtime.block_on(create_executor(&config.get_rpc()?, config.get_wallet()?))?;

        Ok(Self {
            args,
            config,
            executor,
            runtime,
            git,
        })
    }
}

impl RemoteHelper for Evm {
    fn capabilities(&self) -> Vec<&'static str> {
        vec!["*fetch", "*push"]
    }

    fn list(&self, _is_for_push: bool) -> Result<Vec<Reference>, RemoteHelperError> {
        self.runtime.block_on(self.executor.list())
    }

    fn fetch(&self, hash: Hash) -> Result<(), RemoteHelperError> {
        let object = self.runtime.block_on(self.executor.fetch(hash))?;
        self.git.save_object(object)?;
        Ok(())
    }

    fn push(&self, reference: ReferencePush) -> Result<(), RemoteHelperError> {
        let objects = vec![]; // TODO: figure out what objects the remote's missing
        let refs = vec![reference];
        self.runtime.block_on(self.executor.push(objects, refs))
    }
}
