use tempfile::TempDir;

use crate::config::{Config, git::GitConfig};
use std::process::Command;

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
fn git_config() {
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
    assert_eq!(read_value, Some(value.to_string()));

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
    let read_value = config.read(key).expect("failed to read config");
    assert_eq!(read_value, None);
}
