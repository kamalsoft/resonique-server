#[cfg(test)]
mod tests {
    use crate::model::VectorMetadata;
    use crate::storage::search::{Metric, search_segment};
    use crate::storage::segment::Segment;
    use std::collections::HashMap;

    fn get_temp_file() -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("resonique_perf_test_{}", std::process::id()));
        p
    }

    #[test]
    fn test_performance() {
        let path = get_temp_file();
        let mut segment = Segment::create(&path).unwrap();
        let metadata = VectorMetadata {
            tags: vec![],
            timestamp: 0,
            kv: HashMap::new(),
        };

        // Measure insert throughput (insert 100 vectors)
        let start_insert = std::time::Instant::now();
        for i in 0..100 {
            segment.insert(i, &[0.5, 0.5, 0.5, 0.5], &metadata).unwrap();
        }
        let insert_duration = start_insert.elapsed();
        println!("Insert duration for 100 vectors: {:?}", insert_duration);

        // Measure search latency
        let start_search = std::time::Instant::now();
        let results =
            search_segment(&mut segment, &[0.5, 0.5, 0.5, 0.5], 5, Metric::Cosine, None).unwrap();
        let search_duration = start_search.elapsed();
        println!("Search duration: {:?}", search_duration);

        assert_eq!(results.len(), 5);
        let _ = std::fs::remove_file(path);
    }
}
