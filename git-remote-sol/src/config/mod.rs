use std::{error::Error, process::Command};

#[cfg(test)]
mod tests;

pub trait Config {
  fn read(&self, key: &str) -> Result<Option<String>, Box<dyn Error>>;
}

pub struct GitConfig {}

impl GitConfig {
  pub fn new() -> Self {
    Self{}
  }
}

impl Config for GitConfig {
  fn read(&self, key: &str) -> Result<Option<String>, Box<dyn Error>> {
    let cmd = Command::new("git")
      .arg("config")
      .arg("--get")
      .arg(key)
      .output()?;

    let value = String::from_utf8(cmd.stdout)?;
    let trimmed = value.trim();
  
    match value.is_empty() {
      true => Ok(None),
      false => Ok(Some(trimmed.to_string())),
    }
  }
}
