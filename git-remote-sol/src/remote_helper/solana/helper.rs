use crate::config::Config;
use crate::remote_helper::solana::config::SolanaConfig;
use crate::remote_helper::{RemoteHelper, RemoteHelperError, Reference};

pub struct Solana {
  config: SolanaConfig,
}

impl Solana {
  pub fn new(config: Box<dyn Config>) -> Self {
      Self { config: SolanaConfig::new(config) }
  }
}

impl RemoteHelper for Solana {
  fn capabilities(&self) -> Vec<&'static str> {
      vec!["fetch", "push"]
  }

  fn list(&self) -> Result<Vec<Reference>, RemoteHelperError> {
      Ok(vec![])
  }   
}
