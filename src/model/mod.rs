use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub storage_root: String,
    pub collections: Vec<CollectionManifest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionManifest {
    pub name: String,
    pub partitions: Vec<PartitionConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PartitionConfig {
    pub name: String,
    pub hash_range: (u64, u64),
}

#[derive(Debug, Clone)]
pub struct Collection {
    pub name: String,
    pub partitions: Vec<Partition>,
}

#[derive(Debug, Clone)]
pub struct Partition {
    pub name: String,
    pub hash_range: (u64, u64),
}
impl Collection {
    pub fn from_manifest(m: &CollectionManifest) -> Self {
        let partitions = m
            .partitions
            .iter()
            .map(|p| Partition {
                name: p.name.clone(),
                hash_range: p.hash_range,
            })
            .collect();

        Self {
            name: m.name.clone(),
            partitions,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct VectorMetadata {
    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(default)]
    pub timestamp: u64,

    #[serde(default)]
    pub kv: std::collections::HashMap<String, String>,
}
