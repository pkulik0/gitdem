use crate::remote_helper::RemoteHelperError;
use regex::Regex;
use std::{fmt, sync::LazyLock};

static HASH_REGEX_SHA1: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[0-9a-f]{40}$").expect("failed to create sha1 regex"));
static HASH_REGEX_SHA256: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[0-9a-f]{64}$").expect("failed to create sha256 regex"));

#[derive(Clone, Debug, PartialEq)]
pub enum Hash {
    Sha1(String),
    Sha256(String),
}

impl Hash {
    pub fn from_str(s: &str) -> Result<Self, RemoteHelperError> {
        if HASH_REGEX_SHA1.is_match(s) {
            Ok(Self::Sha1(s.to_string()))
        } else if HASH_REGEX_SHA256.is_match(s) {
            Ok(Self::Sha256(s.to_string()))
        } else {
            Err(RemoteHelperError::InvalidHash(s.to_string()))
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Sha1(s) => s.clone(),
            Self::Sha256(s) => s.clone(),
        }
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[test]
fn test_hash() {
    let hash_str = "4e1243bd22c66e76c2ba9eddc1f91394e57f9f83";
    let hash = Hash::from_str(hash_str).expect("failed to create hash");
    assert_eq!(hash, Hash::Sha1(hash_str.to_string()));

    let hash_str = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";
    let hash = Hash::from_str(hash_str).expect("failed to create hash");
    assert_eq!(hash, Hash::Sha256(hash_str.to_string()));

    let hash_str = "4e1243bd22c66e.6c2ba9eddc1f91394e57f9f83";
    let hash = Hash::from_str(hash_str).expect_err("should fail");
    assert_eq!(hash, RemoteHelperError::InvalidHash(hash_str.to_string()));

    let hash_str = "abc";
    let hash = Hash::from_str(hash_str).expect_err("should fail");
    assert_eq!(hash, RemoteHelperError::InvalidHash(hash_str.to_string()));
}
