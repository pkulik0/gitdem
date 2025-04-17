use crate::core::remote_helper::error::RemoteHelperError;
use mockall::automock;
use std::env::VarError;

#[automock]
pub trait KeyValueSource {
    fn read(&self, key: &str) -> Result<Option<String>, RemoteHelperError>;
}

pub struct EnvSource {}

impl EnvSource {
    pub fn new() -> Self {
        Self {}
    }
}

impl KeyValueSource for EnvSource {
    fn read(&self, key: &str) -> Result<Option<String>, RemoteHelperError> {
        let key = key.to_uppercase().replace('.', "_");
        let key = key.strip_prefix("EVM_").unwrap_or(&key);
        let key = format!("GITDEM_{}", key);

        let value = match std::env::var(key) {
            Ok(value) => value.trim().to_string(),
            Err(VarError::NotPresent) => return Ok(None),
            Err(VarError::NotUnicode(_)) => {
                return Err(RemoteHelperError::Failure {
                    action: "reading environment variable".to_string(),
                    details: Some("non-unicode value".to_string()),
                });
            }
        };

        if value.is_empty() {
            Ok(None)
        } else {
            Ok(Some(value))
        }
    }
}

#[test]
fn test_env_source() {
    let expected_value = "test_value";
    unsafe {
        std::env::set_var("GITDEM_SOME_KEY", expected_value);
    }

    let env_source = EnvSource::new();

    let value = env_source.read("evm.some.key").unwrap();
    assert_eq!(value, Some(expected_value.to_string()));

    let value = env_source.read("some.key").unwrap();
    assert_eq!(value, Some(expected_value.to_string()));

    let value = env_source.read("another.key").unwrap();
    assert_eq!(value, None);

    unsafe {
        std::env::remove_var("GITDEM_SOME_KEY");
    }
}
