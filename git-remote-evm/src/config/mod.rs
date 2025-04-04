use std::error::Error;

#[cfg(any(test, feature = "mock"))]
pub mod mock;

pub mod git;

pub trait Config {
  fn read(&self, key: &str) -> Result<Option<String>, Box<dyn Error>>;
}
