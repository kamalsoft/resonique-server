use crate::model::{Collection, Partition};
use crate::server::collection::CollectionManager;
use crate::storage::StorageEngine;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

pub fn temporary_storage() -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    std::env::temp_dir().join(format!("resonique-test-{suffix}"))
}

pub fn manager() -> (Arc<Mutex<CollectionManager>>, PathBuf) {
    let root = temporary_storage();
    let storage = StorageEngine::new(&root).unwrap();
    storage.init().unwrap();

    let collection = Collection {
        name: "default".into(),
        partitions: vec![Partition {
            name: "p0".into(),
            hash_range: (0, u64::MAX),
            node_id: "node-0".into(),
        }],
    };

    let manager = CollectionManager::new(&storage, vec![collection]).unwrap();

    (Arc::new(Mutex::new(manager)), root)
}
