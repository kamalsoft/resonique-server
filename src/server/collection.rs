use anyhow::Result;
use std::collections::HashMap;

use crate::model::Collection;
use crate::storage::StorageEngine;
use crate::storage::segment::Segment;
use crate::storage::wal::Wal;

#[allow(dead_code)]
pub struct PartitionState {
    pub name: String,
    pub hash_range: (u64, u64),
    pub segment: Segment,
    pub wal: Wal,
}

#[allow(dead_code)]
pub struct CollectionState {
    pub name: String,
    pub partitions: Vec<PartitionState>,
}

pub struct CollectionManager {
    pub collections: HashMap<String, CollectionState>,
}

impl CollectionManager {
    pub fn new(engine: &StorageEngine, collections_config: Vec<Collection>) -> Result<Self> {
        let mut collections = HashMap::new();

        for col in collections_config {
            let mut partitions = Vec::new();

            for part in col.partitions {
                let segment_filename = format!("{}_{}.segment", col.name, part.name);
                let wal_filename = format!("{}_{}.wal", col.name, part.name);

                let segment_path = engine.resolve(&segment_filename);
                let wal_path = engine.resolve(&wal_filename);

                // Open existing segment or create a new one
                let mut segment = Segment::open(&segment_path)?;
                let wal = Wal::open(&wal_path)?;

                // Replay WAL to rebuild in-memory index
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

        Ok(Self { collections })
    }

    pub fn route_partition<'a>(
        &'a mut self,
        collection_name: &str,
        vector_id: u64,
    ) -> Result<&'a mut PartitionState> {
        let col = self
            .collections
            .get_mut(collection_name)
            .ok_or_else(|| anyhow::anyhow!("Collection not found: {}", collection_name))?;

        // Simple hash function for routing
        let h = vector_id; // Given u64 id, hash(id) % u64::MAX is basically the id itself in our sandbox

        for part in &mut col.partitions {
            if h >= part.hash_range.0 && h <= part.hash_range.1 {
                return Ok(part);
            }
        }

        Err(anyhow::anyhow!(
            "No partition found for vector_id {}",
            vector_id
        ))
    }
}
