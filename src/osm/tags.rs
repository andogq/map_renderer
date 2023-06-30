use osmpbf::{DenseTagIter, TagIter};
use std::collections::HashMap;

pub struct Tags(HashMap<String, String>);

impl Tags {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.0.get(key)
    }

    pub fn contains(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }
}

impl From<TagIter<'_>> for Tags {
    fn from(tags: TagIter<'_>) -> Self {
        Self(
            tags.map(|(key, value)| (key.to_string(), value.to_string()))
                .collect(),
        )
    }
}

impl From<DenseTagIter<'_>> for Tags {
    fn from(tags: DenseTagIter<'_>) -> Self {
        Self(
            tags.map(|(key, value)| (key.to_string(), value.to_string()))
                .collect(),
        )
    }
}
