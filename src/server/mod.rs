use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod collection;
pub mod http;

use crate::model::Collection;
use crate::storage::StorageEngine;
use crate::util::load_config;
use collection::CollectionManager;

pub async fn run() -> Result<()> {
    crate::util::init_logging();
    tracing::info!("🚀 Resonique Server Starting...");

    let config = load_config("config.json")?;
    tracing::info!("📄 Loaded config: {:?}", config);

    let storage = StorageEngine::new(&config.storage_root)?;
    storage.init()?;
    tracing::info!("📁 Storage root initialized at {}", config.storage_root);

    let collections = config
        .collections
        .iter()
        .map(Collection::from_manifest)
        .collect();
    let mut manager =
        CollectionManager::new_with_node_id(&storage, collections, config.node_id.clone())?;

    manager.set_nodes(config.nodes);

    let shared_manager = Arc::new(Mutex::new(manager));

    let mcp_manager = shared_manager.clone();
    tokio::spawn(async move {
        let _ = crate::mcp::start(mcp_manager).await;
    });

    http::start_http_api(shared_manager).await;

    Ok(())
}
