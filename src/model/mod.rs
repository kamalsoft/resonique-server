use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeConfig {
    pub id: String,
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub storage_root: String,
    #[serde(default = "default_node_id")]
    pub node_id: String,
    #[serde(default)]
    pub nodes: Vec<NodeConfig>,
    pub collections: Vec<CollectionManifest>,
}

fn default_node_id() -> String {
    "node-0".to_owned()
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
    #[serde(default = "default_node_id")]
    pub node_id: String,
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
    pub node_id: String,
}
impl Collection {
    pub fn from_manifest(m: &CollectionManifest) -> Self {
        let partitions = m
            .partitions
            .iter()
            .map(|p| Partition {
                name: p.name.clone(),
                hash_range: p.hash_range,
                node_id: p.node_id.clone(),
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
