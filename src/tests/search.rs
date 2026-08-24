use crate::model::VectorMetadata;
use crate::storage::search::{Metric, QueryFilter, search_segment};
use crate::storage::segment::Segment;
use std::collections::HashMap;

fn path(name: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    let root =
        std::env::temp_dir().join(format!("resonique-search-{}-{}", std::process::id(), name));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    (root.clone(), root.join("data.segment"))
}

fn metadata(tag: &str, key: &str, value: &str) -> VectorMetadata {
    VectorMetadata {
        tags: vec![tag.into()],
        timestamp: 1,
        kv: HashMap::from([(key.into(), value.into())]),
    }
}

#[test]
fn searches_with_cosine_and_l2_metrics() {
    let (root, segment_path) = path("metrics");
    let mut segment = Segment::create(&segment_path).unwrap();

    segment
        .insert(1, &[1.0, 0.0], &metadata("red", "type", "a"))
        .unwrap();
    segment
        .insert(2, &[0.0, 1.0], &metadata("blue", "type", "b"))
        .unwrap();

    let cosine = search_segment(&mut segment, &[1.0, 0.0], 2, Metric::Cosine, None).unwrap();
    assert_eq!(cosine[0].vector_id, 1);
    assert_eq!(cosine.len(), 2);

    let l2 = search_segment(&mut segment, &[1.0, 0.0], 1, Metric::L2, None).unwrap();
    assert_eq!(l2[0].vector_id, 1);

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn applies_tag_and_metadata_filters() {
    let (root, segment_path) = path("filters");
    let mut segment = Segment::create(&segment_path).unwrap();

    segment
        .insert(1, &[1.0, 0.0], &metadata("red", "type", "a"))
        .unwrap();
    segment
        .insert(2, &[0.0, 1.0], &metadata("blue", "type", "b"))
        .unwrap();

    let tag_filter = QueryFilter {
        tag: Some("red".into()),
        metadata_key: None,
        metadata_val: None,
    };
    assert_eq!(
        search_segment(
            &mut segment,
            &[1.0, 0.0],
            10,
            Metric::Cosine,
            Some(tag_filter)
        )
        .unwrap()
        .len(),
        1
    );

    let metadata_filter = QueryFilter {
        tag: None,
        metadata_key: Some("type".into()),
        metadata_val: Some("b".into()),
    };
    let results = search_segment(
        &mut segment,
        &[1.0, 0.0],
        10,
        Metric::Cosine,
        Some(metadata_filter),
    )
    .unwrap();

    assert_eq!(results[0].vector_id, 2);

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn cosine_search_handles_zero_norm_query() {
    let (root, segment_path) = path("zero-query");
    let mut segment = Segment::create(&segment_path).unwrap();

    segment
        .insert(1, &[1.0, 0.0], &metadata("tag", "kind", "value"))
        .unwrap();

    let results = search_segment(&mut segment, &[0.0, 0.0], 1, Metric::Cosine, None).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].score, 0.0);

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn metadata_filter_without_value_requires_key_presence() {
    let (root, segment_path) = path("metadata-key");
    let mut segment = Segment::create(&segment_path).unwrap();

    segment
        .insert(1, &[1.0, 0.0], &metadata("tag", "kind", "value"))
        .unwrap();
    segment
        .insert(
            2,
            &[0.0, 1.0],
            &VectorMetadata {
                tags: vec!["tag".into()],
                timestamp: 1,
                kv: std::collections::HashMap::new(),
            },
        )
        .unwrap();

    let filter = QueryFilter {
        tag: None,
        metadata_key: Some("kind".into()),
        metadata_val: None,
    };

    let results =
        search_segment(&mut segment, &[1.0, 0.0], 10, Metric::Cosine, Some(filter)).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].vector_id, 1);

    let _ = std::fs::remove_dir_all(root);
}
