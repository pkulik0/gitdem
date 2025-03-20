use crate::config::{Config, git::GitConfig};
use std::process::Command;

#[test]
fn git_config() {
    let key = "some.key";
    let value = "123456";
    let config = GitConfig::new();

    Command::new("git")
        .arg("config")
        .arg(key)
        .arg(value)
        .output()
        .expect("failed to run git config");
    let read_value = config.read(key).expect("failed to read config");
    assert_eq!(read_value, Some(value.to_string()));

    Command::new("git")
        .arg("config")
        .arg("--unset")
        .arg(key)
        .output()
        .expect("failed to run git config");
    let read_value = config.read(key).expect("failed to read config");
    assert_eq!(read_value, None);
}
