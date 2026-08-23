use anyhow::Result;
use std::collections::HashMap;

use crate::model::{Collection, NodeConfig};
use crate::storage::StorageEngine;
use crate::storage::segment::Segment;
use crate::storage::wal::Wal;

#[allow(dead_code)]
pub struct PartitionState {
    pub name: String,
    pub hash_range: (u64, u64),
    pub node_id: String,
    pub segment: Segment,
    pub wal: Wal,
}

#[allow(dead_code)]
pub struct CollectionState {
    pub name: String,
    pub partitions: Vec<PartitionState>,
}

pub struct CollectionManager {
    pub node_id: String,
    pub nodes: HashMap<String, String>,
    pub collections: HashMap<String, CollectionState>,
}

impl CollectionManager {
    pub fn set_nodes(&mut self, nodes: Vec<NodeConfig>) {
        self.nodes = nodes
            .into_iter()
            .map(|node| (node.id, node.address))
            .collect();
    }

    #[allow(dead_code)]
    pub fn new(engine: &StorageEngine, collections_config: Vec<Collection>) -> Result<Self> {
        Self::new_with_node_id(engine, collections_config, "node-0")
    }

    pub fn new_with_node_id(
        engine: &StorageEngine,
        collections_config: Vec<Collection>,
        node_id: impl Into<String>,
    ) -> Result<Self> {
        let node_id = node_id.into();
        let mut collections = HashMap::new();

        for col in collections_config {
            let mut partitions = Vec::new();

            for part in col.partitions {
                let segment_filename = format!("{}_{}.segment", col.name, part.name);
                let wal_filename = format!("{}_{}.wal", col.name, part.name);

                let segment_path = engine.resolve(&segment_filename);
                let wal_path = engine.resolve(&wal_filename);
                let mut segment = Segment::open(&segment_path)?;
                let wal = Wal::open(&wal_path)?;

                let records = Wal::replay(&wal_path)?;
                let mut current_offset = segment.header.payload_offset;

                for record in records {
                    let payload_len = record.payload.len() as u32;
                    let metadata_bytes = serde_json::to_vec(&record.metadata)?;
                    let metadata_len = metadata_bytes.len() as u32;

                    segment
                        .index
                        .push(crate::storage::segment::SegmentIndexEntry {
                            vector_id: record.vector_id,
                            offset: current_offset,
                            length: payload_len,
                            metadata_offset: current_offset + payload_len as u64,
                        });
                    segment.header.entry_count += 1;
                    current_offset += (payload_len + metadata_len) as u64;
                }

                partitions.push(PartitionState {
                    name: part.name,
                    hash_range: part.hash_range,
                    node_id: part.node_id,
                    segment,
                    wal,
                });
            }

            collections.insert(
                col.name.clone(),
                CollectionState {
                    name: col.name,
                    partitions,
                },
            );
        }

        Ok(Self {
            node_id,
            nodes: HashMap::new(),
            collections,
        })
    }

    pub fn route_partition<'a>(
        &'a mut self,
        collection_name: &str,
        vector_id: u64,
    ) -> Result<&'a mut PartitionState> {
        let node_id = self.node_id.clone();
        let col = self
            .collections
            .get_mut(collection_name)
            .ok_or_else(|| anyhow::anyhow!("Collection not found: {collection_name}"))?;

        col.partitions
            .iter_mut()
            .find(|part| {
                part.node_id == node_id
                    && vector_id >= part.hash_range.0
                    && vector_id <= part.hash_range.1
            })
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Partition for vector_id {vector_id} is not owned by node {node_id}"
                )
            })
    }

    #[allow(dead_code)]
    pub fn partition_owner(
        &self,
        collection_name: &str,
        vector_id: u64,
    ) -> Result<(String, String)> {
        let collection = self
            .collections
            .get(collection_name)
            .ok_or_else(|| anyhow::anyhow!("Collection not found: {collection_name}"))?;

        let partition = collection
            .partitions
            .iter()
            .find(|partition| {
                vector_id >= partition.hash_range.0 && vector_id <= partition.hash_range.1
            })
            .ok_or_else(|| anyhow::anyhow!("No partition found for vector_id {vector_id}"))?;

        let address = self
            .nodes
            .get(&partition.node_id)
            .ok_or_else(|| anyhow::anyhow!("Unknown node: {}", partition.node_id))?;

        Ok((partition.node_id.clone(), address.clone()))
    }
}
