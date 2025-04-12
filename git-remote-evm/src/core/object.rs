use super::hash::Hash;

#[derive(Debug, PartialEq, Clone)]
pub struct Object {
    pub hash: Hash,
    pub data: Vec<u8>,
}

impl Object {
    pub fn new(hash: Hash, data: Vec<u8>) -> Self {
        Self { hash, data }
    }
}
