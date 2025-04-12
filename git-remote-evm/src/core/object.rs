use super::hash::Hash;
use crate::core::remote_helper::error::RemoteHelperError;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone)]
pub enum ObjectKind {
    Blob,
    Tree,
    Commit,
    Tag,
}

impl std::fmt::Display for ObjectKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectKind::Blob => write!(f, "blob"),
            ObjectKind::Tree => write!(f, "tree"),
            ObjectKind::Commit => write!(f, "commit"),
            ObjectKind::Tag => write!(f, "tag"),
        }
    }
}

impl FromStr for ObjectKind {
    type Err = RemoteHelperError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "blob" => ObjectKind::Blob,
            "tree" => ObjectKind::Tree,
            "commit" => ObjectKind::Commit,
            "tag" => ObjectKind::Tag,
            _ => {
                return Err(RemoteHelperError::Invalid {
                    what: "object kind".to_string(),
                    value: s.to_string(),
                });
            }
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Object {
    pub kind: ObjectKind,
    pub data: Vec<u8>,
}

impl Object {
    pub fn new(kind: ObjectKind, data: Vec<u8>) -> Self {
        Self { kind, data }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        data.extend_from_slice(self.kind.to_string().as_bytes());
        data.push(b' ');
        data.extend_from_slice(self.data.len().to_string().as_bytes());
        data.push(b'\0');
        data.extend_from_slice(&self.data);
        data
    }

    pub fn hash(&self, is_sha256: bool) -> Hash {
        let data = self.serialize();
        if is_sha256 {
            Hash::from_data_sha256(&data).expect("creating hash from a valid object failed")
        } else {
            Hash::from_data_sha1(&data).expect("creating hash from a valid object failed")
        }
    }

    pub fn deserialize(input: &[u8]) -> Result<Self, RemoteHelperError> {
        let parts = input.splitn(2, |b| *b == b'\0').collect::<Vec<_>>();
        if parts.len() != 2 {
            return Err(RemoteHelperError::Invalid {
                what: "object".to_string(),
                value: String::from_utf8_lossy(input).to_string(),
            });
        }

        let header =
            String::from_utf8(parts[0].to_vec()).map_err(|e| RemoteHelperError::Invalid {
                what: "object header".to_string(),
                value: e.to_string(),
            })?;
        let data = parts[1];

        let parts = header.splitn(2, |b| b == ' ').collect::<Vec<_>>();
        if parts.len() != 2 {
            return Err(RemoteHelperError::Invalid {
                what: "object header".to_string(),
                value: header,
            });
        }

        let kind = ObjectKind::from_str(&parts[0])?;
        let size = parts[1]
            .parse::<usize>()
            .map_err(|e| RemoteHelperError::Invalid {
                what: "object size".to_string(),
                value: e.to_string(),
            })?;

        if size != data.len() {
            return Err(RemoteHelperError::Invalid {
                what: format!("object size: {}, expected: {}", data.len(), size),
                value: String::from_utf8_lossy(input).to_string(),
            });
        }

        Ok(Self {
            kind,
            data: data.to_vec(),
        })
    }
}

#[test]
fn test_object_deserialize() {
    let object = Object::deserialize(b"blob 0\0").unwrap();
    assert_eq!(object.kind, ObjectKind::Blob);
    assert_eq!(object.data, vec![] as Vec<u8>);

    let object = Object::deserialize(b"blob 4\0test").unwrap();
    assert_eq!(object.kind, ObjectKind::Blob);
    assert_eq!(object.data, b"test");
}

#[test]
fn test_object_serialize() {
    let object = Object::new(ObjectKind::Blob, vec![]);
    assert_eq!(object.serialize(), b"blob 0\0");

    let object = Object::new(ObjectKind::Blob, b"test".to_vec());
    assert_eq!(object.serialize(), b"blob 4\0test");
}
