use crate::core::remote_helper::error::RemoteHelperError;
use log::{debug, trace};
use mockall::automock;
use std::path::PathBuf;
use std::process::Command;

#[cfg(test)]
use tempfile::TempDir;

#[automock]
pub trait KeyValueSource {
    fn read(&self, key: &str) -> Result<Option<String>, RemoteHelperError>;
}

pub struct GitConfigSource {
    dir: PathBuf,
}

impl GitConfigSource {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }
}

impl KeyValueSource for GitConfigSource {
    fn read(&self, key: &str) -> Result<Option<String>, RemoteHelperError> {
        trace!("reading git config key: {}", key);
        let cmd = Command::new("git")
            .arg("config")
            .arg("--get")
            .arg(key)
            .current_dir(self.dir.as_path())
            .output()
            .map_err(|e| RemoteHelperError::Failure {
                action: "running git config".to_string(),
                details: Some(e.to_string()),
            })?;

        let value = String::from_utf8(cmd.stdout).map_err(|e| RemoteHelperError::Failure {
            action: "parsing git config output".to_string(),
            details: Some(e.to_string()),
        })?;
        let trimmed = value.trim();

        let result = match value.is_empty() {
            true => None,
            false => Some(trimmed.to_string()),
        };
        debug!("git config {} = {:?}", key, result);
        Ok(result)
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
    let config = GitConfigSource::new(repo_dir.path().to_path_buf());

    let cmd = Command::new("git")
        .arg("config")
        .arg(key)
        .arg(value)
        .current_dir(repo_dir.path())
        .output()
        .expect("failed to run git config");
    if !cmd.status.success() {
        panic!(
            "git config failed: {}",
            String::from_utf8_lossy(&cmd.stderr)
        );
    }
    let read_value = config
        .read(key)
        .expect("failed to read config")
        .expect("doesn't have value");
    assert_eq!(read_value, value.to_string());

    let cmd = Command::new("git")
        .arg("config")
        .arg("--unset")
        .arg(key)
        .current_dir(repo_dir.path())
        .output()
        .expect("failed to run git config");
    if !cmd.status.success() {
        panic!(
            "git config failed: {}",
            String::from_utf8_lossy(&cmd.stderr)
        );
    }
    let read_value = config.read(key).expect("failed to read config");
    assert!(read_value.is_none());
}
