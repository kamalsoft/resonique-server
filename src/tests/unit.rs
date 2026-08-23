#[cfg(test)]
mod tests {
    use crate::model::VectorMetadata;
    use crate::storage::index::SecondaryIndex;
    use crate::storage::search::{Metric, QueryFilter, search_segment};
    use crate::storage::segment::Segment;
    use crate::storage::wal::Wal;
    use std::collections::HashMap;

    fn get_temp_file(suffix: &str) -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "resonique_test_{}_{}{}",
            std::process::id(),
            rand_u64(),
            suffix
        ));
        p
    }

    fn rand_u64() -> u64 {
        use std::collections::hash_map::RandomState;
        use std::hash::{BuildHasher, Hasher};
        let mut hasher = RandomState::new().build_hasher();
        hasher.write_u64(0);
        hasher.finish()
    }

    #[test]
    fn test_segment_creation() {
        let path = get_temp_file(".segment");
        let segment = Segment::create(&path).unwrap();
        assert_eq!(segment.header.magic, 0xDEADBEEF);
        assert_eq!(segment.header.entry_count, 0);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_wal_append() {
        let path = get_temp_file(".wal");
        let mut wal = Wal::open(&path).unwrap();
        let metadata = VectorMetadata {
            tags: vec!["test".to_string()],
            timestamp: 12345,
            kv: HashMap::new(),
        };
        wal.append(1, &[0.1, 0.2], &metadata).unwrap();

        let records = Wal::replay(&path).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].vector_id, 1);
        assert_eq!(records[0].metadata.tags[0], "test");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_segment_insert_and_search() {
        let path = get_temp_file(".segment");
        let mut segment = Segment::create(&path).unwrap();
        let metadata = VectorMetadata {
            tags: vec!["foo".to_string()],
            timestamp: 123,
            kv: HashMap::new(),
        };
        segment.insert(42, &[1.0, 0.0], &metadata).unwrap();

        // Search cosine
        let results = search_segment(&mut segment, &[1.0, 0.0], 5, Metric::Cosine, None).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].vector_id, 42);
        assert!((results[0].score - 1.0).abs() < 1e-5);

        // Search with filter
        let filter = QueryFilter {
            tag: Some("foo".to_string()),
            metadata_key: None,
            metadata_val: None,
        };
        let results_filtered =
            search_segment(&mut segment, &[1.0, 0.0], 5, Metric::Cosine, Some(filter)).unwrap();
        assert_eq!(results_filtered.len(), 1);

        // Search with non-matching filter
        let filter_bad = QueryFilter {
            tag: Some("bar".to_string()),
            metadata_key: None,
            metadata_val: None,
        };
        let results_empty = search_segment(
            &mut segment,
            &[1.0, 0.0],
            5,
            Metric::Cosine,
            Some(filter_bad),
        )
        .unwrap();
        assert_eq!(results_empty.len(), 0);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_secondary_index() {
        let mut index = SecondaryIndex::new();
        index.insert(1, &["tag1".to_string(), "tag2".to_string()]);
        assert_eq!(index.get("tag1").unwrap(), &vec![1]);
        assert_eq!(index.get("tag2").unwrap(), &vec![1]);
        assert!(index.get("tag3").is_none());
    }
}
