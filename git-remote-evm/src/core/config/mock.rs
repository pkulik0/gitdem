use super::Config;
use std::collections::HashMap;

pub struct MockConfig {
    values: HashMap<String, String>,
}

impl MockConfig {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn new_with_values(values: HashMap<String, String>) -> Self {
        Self { values }
    }
}

impl Config for MockConfig {
    fn read(&self, key: &str) -> Option<String> {
        self.values.get(key).cloned()
    }
}
