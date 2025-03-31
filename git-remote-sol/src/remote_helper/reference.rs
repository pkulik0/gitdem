use std::fmt;

// gitremote-helpers.adoc (line 438)
#[derive(Clone)]
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
#[derive(Clone)]
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
#[derive(Clone)]
pub enum Value {
    Hash(String),
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
#[derive(Clone)]
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
