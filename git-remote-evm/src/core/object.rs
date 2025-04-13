use super::hash::Hash;
use crate::core::remote_helper::error::RemoteHelperError;
use std::hash::Hash as StdHash;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone, Eq, StdHash)]
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

#[derive(Debug, PartialEq, Clone, Eq, StdHash)]
pub struct Object {
    kind: ObjectKind,
    data: Vec<u8>,
    related_objects: Vec<Hash>,
}

impl Object {
    pub fn new(kind: ObjectKind, data: Vec<u8>) -> Result<Self, RemoteHelperError> {
        let related_objects = Self::find_related_objects(&kind, &data)?;
        Ok(Self { kind, data, related_objects })
    }

    pub fn get_kind(&self) -> &ObjectKind {
        &self.kind
    }

    pub fn get_data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn get_related_objects(&self) -> &Vec<Hash> {
        &self.related_objects
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

    fn find_related_objects(kind: &ObjectKind, data: &[u8]) -> Result<Vec<Hash>, RemoteHelperError> {
        let data_str = String::from_utf8_lossy(data).to_string();
        match kind {
            ObjectKind::Blob => Ok(vec![]),
            ObjectKind::Tree => {
                let mut related_objects = vec![];
                for line in data_str.lines() {
                    let parts = line.split_whitespace().collect::<Vec<_>>();
                    match parts.len() {
                        3 | 4 => related_objects.push(Hash::from_str(&parts[2])?),
                        _ => return Err(RemoteHelperError::Invalid {
                            what: "object tree".to_string(),
                            value: data_str,
                        })
                    }
                }
                Ok(related_objects)
            },
            ObjectKind::Commit => {
                let mut related_objects = vec![];
                for line in data_str.lines() {
                    let parts = line.split_whitespace().collect::<Vec<_>>();
                    if parts.len() < 2 {
                        return Err(RemoteHelperError::Invalid {
                            what: "object commit".to_string(),
                            value: data_str,
                        });
                    }
                    let kind = parts[0];
                    match kind {
                        "tree" | "parent" => {
                            related_objects.push(Hash::from_str(&parts[1])?);
                        },
                        _ => break,
                    }
                }
                Ok(related_objects)
            },
            ObjectKind::Tag => {
                let lines = data_str.lines().collect::<Vec<_>>();
                if lines.is_empty() {
                    return Err(RemoteHelperError::Invalid {
                        what: "object tag".to_string(),
                        value: data_str,
                    });
                }
                let parts = lines[0].split_whitespace().collect::<Vec<_>>();
                if parts.len() != 2 {
                    return Err(RemoteHelperError::Invalid {
                        what: "object tag".to_string(),
                        value: data_str,
                    });
                }
                let kind = parts[0];
                if kind != "object" {
                    return Err(RemoteHelperError::Invalid {
                        what: "object tag".to_string(),
                        value: data_str,
                    });
                }
                let object = Hash::from_str(&parts[1])?;
                Ok(vec![object])
            },
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

        let related_objects = Self::find_related_objects(&kind, &data)?;
        Ok(Self {
            kind,
            data: data.to_vec(),
            related_objects,
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
    let object = Object {
        kind: ObjectKind::Blob,
        data: vec![],
        related_objects: vec![],
    };
    assert_eq!(object.serialize(), b"blob 0\0");

    let object = Object {
        kind: ObjectKind::Blob,
        data: b"test".to_vec(),
        related_objects: vec![],
    };
    assert_eq!(object.serialize(), b"blob 4\0test");
}
