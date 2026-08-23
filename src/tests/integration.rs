#[cfg(test)]
mod tests {
    use crate::model::{Collection, VectorMetadata};
    use crate::server::collection::CollectionManager;
    use crate::storage::StorageEngine;
    use crate::storage::search::Metric;
    use std::collections::HashMap;

    fn get_temp_dir() -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("resonique_int_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&p);
        p
    }

    #[test]
    fn test_e2e_insert_search_and_replay() {
        let temp_root = get_temp_dir();
        let engine = StorageEngine::new(&temp_root).unwrap();
        engine.init().unwrap();

        let cfg = vec![Collection {
            name: "col1".to_string(),
            partitions: vec![crate::model::Partition {
                name: "p0".to_string(),
                hash_range: (0, u64::MAX),
                node_id: "node-0".to_string(),
            }],
        }];

        // Initialize manager
        let mut manager = CollectionManager::new(&engine, cfg.clone()).unwrap();

        let metadata = VectorMetadata {
            tags: vec!["foo".to_string()],
            timestamp: 999,
            kv: HashMap::new(),
        };

        // Route insert
        let part = manager.route_partition("col1", 100).unwrap();
        part.wal.append(100, &[0.5, 0.5], &metadata).unwrap();
        part.segment.insert(100, &[0.5, 0.5], &metadata).unwrap();

        // Query the segment directly
        let results = crate::storage::search::search_segment(
            &mut part.segment,
            &[0.5, 0.5],
            5,
            Metric::Cosine,
            None,
        )
        .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].vector_id, 100);

        // Restart CollectionManager and verify WAL Replay restores index!
        let manager_restarted = CollectionManager::new(&engine, cfg).unwrap();
        let col = manager_restarted.collections.get("col1").unwrap();
        let part_restarted = &col.partitions[0];
        assert_eq!(part_restarted.segment.index.len(), 1);
        assert_eq!(part_restarted.segment.index[0].vector_id, 100);

        let _ = std::fs::remove_dir_all(temp_root);
    }
}
