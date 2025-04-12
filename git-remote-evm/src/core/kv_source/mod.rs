pub mod git_config;
#[cfg(any(test, feature = "mock"))]
pub mod mock;

pub trait KeyValueSource {
    fn read(&self, key: &str) -> Option<String>;
}
