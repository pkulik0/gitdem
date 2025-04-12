use super::{hash::Hash, remote_helper::error::RemoteHelperError};
use std::{fmt, str::FromStr};

// gitremote-helpers.adoc (line 449)
#[derive(Clone, Debug, PartialEq)]
pub enum Keys {
    ObjectFormat,
}

impl fmt::Display for Keys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Keys::ObjectFormat => write!(f, "object-format"),
        }
    }
}

impl FromStr for Keys {
    type Err = RemoteHelperError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "object-format" {
            Ok(Keys::ObjectFormat)
        } else {
            Err(RemoteHelperError::Failure {
                action: "parsing keys".to_string(),
                details: Some("invalid key".to_string()),
            })
        }
    }
}

// gitremote-helpers.adoc (line 264)
#[derive(Clone, Debug, PartialEq)]
pub enum Reference {
    Normal { name: String, hash: Hash },
    Symbolic { name: String, target: String },
    KeyValue { key: Keys, value: String },
}

impl fmt::Display for Reference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Reference::Normal { name, hash } => write!(f, "{} {}", hash, name),
            Reference::Symbolic { name, target } => write!(f, "@{} {}", target, name),
            Reference::KeyValue { key, value } => write!(f, ":{} {}", key, value),
        }
    }
}

// gitremote-helpers.adoc (line 321)
#[derive(Clone, Debug, PartialEq)]
pub struct Push {
    pub local: String,
    pub remote: String,
    pub is_force: bool,
}

impl fmt::Display for Push {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_force {
            write!(f, "+")?;
        }
        write!(f, "{}:{}", self.local, self.remote)?;
        Ok(())
    }
}

impl Push {
    pub fn new(local: String, remote: String, is_force: bool) -> Self {
        Self {
            local,
            remote,
            is_force,
        }
    }
}
