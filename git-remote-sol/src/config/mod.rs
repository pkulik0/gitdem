use std::error::Error;

#[cfg(test)]
mod tests;
#[cfg(test)]
pub mod mock;

pub mod git;

pub trait Config {
  fn read(&self, key: &str) -> Result<Option<String>, Box<dyn Error>>;
}
