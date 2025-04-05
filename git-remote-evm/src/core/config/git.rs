use super::Config;
use std::process::Command;
use std::path::PathBuf;
#[cfg(test)]
use tempfile::TempDir;

pub struct GitConfig {
  dir: PathBuf,
}

impl GitConfig {
  pub fn new(dir: PathBuf) -> Self {
    Self{dir}
  }
}

impl Config for GitConfig {
  fn read(&self, key: &str) -> Option<String> {
    let cmd = Command::new("git")
      .arg("config")
      .arg("--get")
      .arg(key)
      .current_dir(self.dir.as_path())
      .output()
      .ok()?;

    let value = String::from_utf8(cmd.stdout).ok()?;
    let trimmed = value.trim();
  
    match value.is_empty() {
      true => None,
      false => Some(trimmed.to_string()),
    }
  }
}


#[cfg(test)]
fn prepare_temp_repo() -> TempDir {
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    
    let cmd = Command::new("git")
        .arg("init")
        .current_dir(temp_dir.path().to_path_buf())
        .output()
        .expect("failed to run git init");
    if !cmd.status.success() {
        panic!("git init failed: {}", String::from_utf8_lossy(&cmd.stderr));
    }

    temp_dir
}

#[test]
fn test_git_config() {
    let repo_dir = prepare_temp_repo();

    let _path = repo_dir.path().to_path_buf();

    let key = "some.key";
    let value = "123456";
    let config = GitConfig::new(repo_dir.path().to_path_buf());

    let cmd = Command::new("git")
        .arg("config")
        .arg(key)
        .arg(value)
        .current_dir(repo_dir.path())
        .output()
        .expect("failed to run git config");
    if !cmd.status.success() {
        panic!("git config failed: {}", String::from_utf8_lossy(&cmd.stderr));
    }
    let read_value = config.read(key).expect("failed to read config");
    assert_eq!(read_value, value.to_string());

    let cmd = Command::new("git")
        .arg("config")
        .arg("--unset")
        .arg(key)
        .current_dir(repo_dir.path())
        .output()
        .expect("failed to run git config");
    if !cmd.status.success() {
        panic!("git config failed: {}", String::from_utf8_lossy(&cmd.stderr));
    }
    assert!(config.read(key).is_none());
}
