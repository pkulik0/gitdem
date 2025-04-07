use super::hash::Hash;
use std::fmt;

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

// gitremote-helpers.adoc (line 264)
#[derive(Clone, Debug, PartialEq)]
pub enum Reference {
    Normal {
        name: String,
        hash: Hash,
    },
    Symbolic {
        name: String,
        target: String,
    },
    KeyValue {
        key: Keys,
        value: String
    },
    Unknown,
}

impl fmt::Display for Reference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Reference::Normal{ name, hash } => write!(f, "{} {}", hash, name),
            Reference::Symbolic{name, target} => write!(f, "@{} {}", target, name),
            Reference::KeyValue{key, value} => write!(f, ":{} {}", key, value),
            Reference::Unknown => write!(f, "?"),
        }
    }
}

// gitremote-helpers.adoc (line 321)
#[derive(Clone, Debug, PartialEq)]
pub struct ReferencePush {
    pub src: String,
    pub dest: String,
    pub is_force: bool,
}

impl fmt::Display for ReferencePush {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_force {
            write!(f, "+")?;
        }
        write!(f, "{}:{}", self.src, self.dest)?;
        Ok(())
    }
}

impl ReferencePush {
    pub fn new(src: String, dest: String, is_force: bool) -> Self {
        Self {
            src,
            dest,
            is_force,
        }
    }
}
