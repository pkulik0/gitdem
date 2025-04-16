use alloy::primitives::FixedBytes;
use regex::Regex;
use std::{fmt, hash::Hash as StdHash, str::FromStr, sync::LazyLock};

use super::remote_helper::error::RemoteHelperError;

static HASH_REGEX_SHA1: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[0-9a-f]{40}$").expect("failed to create sha1 regex"));
static HASH_REGEX_SHA256: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[0-9a-f]{64}$").expect("failed to create sha256 regex"));

#[derive(Clone, Debug, PartialEq, Eq, StdHash)]
pub enum Hash {
    Sha1(String),
    Sha256(String),
}

impl Hash {
    pub fn is_sha256(&self) -> bool {
        matches!(self, Self::Sha256(_))
    }

    pub fn empty(is_sha256: bool) -> Self {
        if is_sha256 {
            Self::Sha256("0".repeat(64).to_string())
        } else {
            Self::Sha1("0".repeat(40).to_string())
        }
    }

    pub fn padded(&self) -> String {
        // pad with trailing zeros to make it 64 characters long
        match self {
            Self::Sha1(s) => s.clone() + &"0".repeat(64 - s.len()),
            Self::Sha256(s) => s.clone(),
        }
    }

    pub fn from_data(data: &[u8], is_sha256: bool) -> Result<Self, RemoteHelperError> {
        if is_sha256 {
            use sha2::{Digest, Sha256};
            let hash = Sha256::digest(data);
            Ok(Self::Sha256(hex::encode(hash)))
        } else {
            use sha1::{Digest, Sha1};
            let hash = Sha1::digest(data);
            Ok(Self::Sha1(hex::encode(hash)))
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Sha1(_) => self == &Hash::empty(false),
            Self::Sha256(_) => self == &Hash::empty(true),
        }
    }
}

impl FromStr for Hash {
    type Err = RemoteHelperError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let no_padding = s.strip_suffix(&"0".repeat(24)).unwrap_or(s);

        if HASH_REGEX_SHA1.is_match(no_padding) {
            Ok(Self::Sha1(no_padding.to_string()))
        } else if HASH_REGEX_SHA256.is_match(no_padding) {
            Ok(Self::Sha256(no_padding.to_string()))
        } else {
            Err(RemoteHelperError::Failure {
                action: "parsing hash".to_string(),
                details: Some(format!("invalid hash: {:?}", no_padding)),
            })
        }
    }
}

impl From<FixedBytes<32>> for Hash {
    fn from(value: FixedBytes<32>) -> Self {
        let str = value.to_string()[2..].to_string();
        Self::from_str(&str).expect("the hash should be valid")
    }
}

impl TryFrom<&[u8]> for Hash {
    type Error = RemoteHelperError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let str = hex::encode(value);
        Hash::from_str(&str)
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
