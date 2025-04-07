use regex::Regex;
use std::{fmt, str::FromStr, sync::LazyLock};

use super::remote_helper::error::RemoteHelperError;

static HASH_REGEX_SHA1: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[0-9a-f]{40}$").expect("failed to create sha1 regex"));
static HASH_REGEX_SHA256: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[0-9a-f]{64}$").expect("failed to create sha256 regex"));

#[derive(Clone, Debug, PartialEq)]
pub enum Hash {
    Sha1(String),
    Sha256(String),
}

impl FromStr for Hash {
    type Err = RemoteHelperError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if HASH_REGEX_SHA1.is_match(s) {
            Ok(Self::Sha1(s.to_string()))
        } else if HASH_REGEX_SHA256.is_match(s) {
            Ok(Self::Sha256(s.to_string()))
        } else {
            Err(RemoteHelperError::Failure {
                action: "parsing hash".to_string(),
                details: Some("invalid hash".to_string()),
            })
        }
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Sha1(s) => s,
            Self::Sha256(s) => s,
        };
        write!(f, "{}", s)
    }
}

#[test]
fn test_hash() {
    let hash_str = "4e1243bd22c66e76c2ba9eddc1f91394e57f9f83";
    let hash = Hash::from_str(hash_str).expect("should succeed");
    assert_eq!(hash, Hash::Sha1(hash_str.to_string()));

    let hash_str = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";
    let hash = Hash::from_str(hash_str).expect("should succeed");
    assert_eq!(hash, Hash::Sha256(hash_str.to_string()));

    let hash_str = "4e1243bd22c66e.6c2ba9eddc1f91394e57f9f83";
    Hash::from_str(hash_str).expect_err("should fail");

    let hash_str = "abc";
    Hash::from_str(hash_str).expect_err("should fail");
}
