use crate::config::Config;
use std::collections::HashMap;
use std::error::Error;

pub struct MockConfig {
    values: HashMap<String, String>,
}

impl MockConfig {
    pub fn new() -> Self {
        Self { values: HashMap::new() }
    }

    pub fn new_with_values(values: HashMap<String, String>) -> Self {
        Self { values }
    }
}

impl Config for MockConfig {
    fn read(&self, key: &str) -> Result<Option<String>, Box<dyn Error>> {
        Ok(self.values.get(key).cloned())
    }
}
