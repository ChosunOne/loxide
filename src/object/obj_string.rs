use std::{fmt::Display, hash::Hash};

use super::HeapSize;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ObjString {
    pub chars: String,
    pub hash: u32,
}

impl Hash for ObjString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u32(self.hash);
    }
}

impl Eq for ObjString {}

impl HeapSize for ObjString {
    fn size(&self) -> usize {
        self.chars.len() + size_of::<String>() + size_of::<u32>()
    }
}

impl From<&str> for ObjString {
    fn from(value: &str) -> Self {
        let hash = hash_str(value);
        Self {
            chars: value.into(),
            hash,
        }
    }
}

impl From<String> for ObjString {
    fn from(value: String) -> Self {
        let hash = hash_str(&value);
        Self { chars: value, hash }
    }
}

impl Display for ObjString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.chars)
    }
}

fn hash_str(value: &str) -> u32 {
    let mut hash = 2166136261u32;
    for c in value.chars() {
        hash ^= c as u32;
        hash = hash.wrapping_mul(16777619);
    }

    hash
}
