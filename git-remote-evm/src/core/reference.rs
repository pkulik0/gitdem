use super::hash::Hash;
use std::fmt;

// gitremote-helpers.adoc (line 438)
#[derive(Clone, Debug, PartialEq)]
pub enum Attribute {
    Unchanged,
}

impl fmt::Display for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Attribute::Unchanged => write!(f, "unchanged"),
        }
    }
}

// gitremote-helpers.adoc (line 449)
#[derive(Clone, Debug, PartialEq)]
pub enum Keyword {
    ObjectFormat(String),
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Keyword::ObjectFormat(format) => write!(f, "object-format {}", format),
        }
    }
}

// gitremote-helpers.adoc (line 265)
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Hash(Hash),
    SymRef(String),
    KeyValue(Keyword),
    Unknown,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Hash(hash) => write!(f, "{}", hash),
            Value::SymRef(symref) => write!(f, "@{}", symref),
            Value::KeyValue(keyword) => write!(f, ":{}", keyword),
            Value::Unknown => write!(f, "?"),
        }
    }
}

// gitremote-helpers.adoc (line 264)
#[derive(Clone, Debug, PartialEq)]
pub struct Reference {
    pub value: Value,
    pub name: String,
    pub attributes: Vec<Attribute>,
}

impl fmt::Display for Reference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)?;
        if !self.name.is_empty() {
            write!(f, " {}", self.name)?;
        }
        for attr in self.attributes.iter() {
            write!(f, " {}", attr)?;
        }
        Ok(())
    }
}

impl Reference {
    pub fn new_with_hash(name: String, hash: Hash) -> Self {
        Self {
            value: Value::Hash(hash),
            name,
            attributes: vec![],
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
        Self { src, dest, is_force }
    }
}
