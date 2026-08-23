use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct SecondaryIndex {
    tags: HashMap<String, Vec<u64>>,
}

// This index is part of the storage API and is intentionally retained until
// the query path consumes it.
#[allow(dead_code)]
impl SecondaryIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, vector_id: u64, tags: &[String]) {
        for tag in tags {
            self.tags.entry(tag.clone()).or_default().push(vector_id);
        }
    }

    pub fn get(&self, tag: &str) -> Option<&Vec<u64>> {
        self.tags.get(tag)
    }
}
