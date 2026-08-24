use crate::model::{Collection, CollectionManifest, NodeConfig, PartitionConfig, VectorMetadata};

#[test]
fn vector_metadata_defaults() {
    let metadata = VectorMetadata::default();

    assert!(metadata.tags.is_empty());
    assert_eq!(metadata.timestamp, 0);
    assert!(metadata.kv.is_empty());
}

#[test]
fn server_config_applies_default_node_id() {
    let config: crate::model::ServerConfig = serde_json::from_value(serde_json::json!({
        "storage_root": "/tmp/resonique",
        "collections": []
    }))
    .unwrap();

    assert_eq!(config.node_id, "node-0");
    assert!(config.nodes.is_empty());
}

#[test]
fn partition_config_applies_default_node_id() {
    let partition: PartitionConfig = serde_json::from_value(serde_json::json!({
        "name": "p0",
        "hash_range": [0, 10]
    }))
    .unwrap();

    assert_eq!(partition.node_id, "node-0");
}

#[test]
fn collection_is_created_from_manifest() {
    let manifest = CollectionManifest {
        name: "docs".into(),
        partitions: vec![PartitionConfig {
            name: "p0".into(),
            hash_range: (10, 20),
            node_id: "node-1".into(),
        }],
    };

    let collection = Collection::from_manifest(&manifest);

    assert_eq!(collection.name, "docs");
    assert_eq!(collection.partitions.len(), 1);
    assert_eq!(collection.partitions[0].name, "p0");
    assert_eq!(collection.partitions[0].hash_range, (10, 20));
    assert_eq!(collection.partitions[0].node_id, "node-1");
}

#[test]
fn node_config_round_trips_json() {
    let node = NodeConfig {
        id: "node-1".into(),
        address: "127.0.0.1:3001".into(),
    };

    let encoded = serde_json::to_value(&node).unwrap();
    let decoded: NodeConfig = serde_json::from_value(encoded).unwrap();

    assert_eq!(decoded.id, "node-1");
    assert_eq!(decoded.address, "127.0.0.1:3001");
}
