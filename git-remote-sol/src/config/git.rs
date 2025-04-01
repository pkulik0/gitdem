use crate::config::Config;
use std::error::Error;
use std::process::Command;
use std::path::PathBuf;

pub struct GitConfig {
  dir: PathBuf,
}

impl GitConfig {
  pub fn new(dir: PathBuf) -> Self {
    Self{dir}
  }
}

impl Config for GitConfig {
  fn read(&self, key: &str) -> Result<Option<String>, Box<dyn Error>> {
    let cmd = Command::new("git")
      .arg("config")
      .arg("--get")
      .arg(key)
      .current_dir(self.dir.as_path())
      .output()?;

    let value = String::from_utf8(cmd.stdout)?;
    let trimmed = value.trim();
  
    match value.is_empty() {
      true => Ok(None),
      false => Ok(Some(trimmed.to_string())),
    }
  }
}
