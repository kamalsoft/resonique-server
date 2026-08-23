use axum::{
    Router,
    extract::{Json, Path, State},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::Mutex;

pub static INSERT_COUNT: AtomicU64 = AtomicU64::new(0);
pub static SEARCH_COUNT: AtomicU64 = AtomicU64::new(0);

use crate::server::collection::CollectionManager;

#[derive(Deserialize)]
pub struct InsertRequest {
    pub collection: String,
    pub vector_id: u64,
    pub vector: Vec<f32>,
    pub metadata: Option<crate::model::VectorMetadata>,
}

#[derive(Serialize)]
pub struct InsertResponse {
    pub status: String,
}

#[derive(Deserialize)]
pub struct SearchRequest {
    pub collection: String,
    pub query: Vec<f32>,
    pub top_k: usize,
    pub metric: Option<crate::storage::search::Metric>,
    pub filter: Option<crate::storage::search::QueryFilter>,
}

#[derive(Serialize)]
pub struct SearchResultDto {
    pub vector_id: u64,
    pub score: f32,
    pub metadata: crate::model::VectorMetadata,
}

#[derive(Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResultDto>,
}

type SharedState = Arc<Mutex<CollectionManager>>;

pub async fn start_http_api(manager: SharedState) {
    let app = Router::new()
        .route("/health", get(handle_health))
        .route("/metrics", get(handle_metrics))
        .route("/insert", post(handle_insert))
        .route("/search", post(handle_search))
        .route("/collections", get(handle_collections))
        .route(
            "/collections/:collection/partitions",
            get(handle_partitions),
        )
        .with_state(manager);

    let listener = match tokio::net::TcpListener::bind("127.0.0.1:3000").await {
        Ok(listener) => listener,
        Err(error) => {
            tracing::error!(%error, "failed to bind HTTP API listener");
            return;
        }
    };

    tracing::info!("HTTP API listening on {}", listener.local_addr().unwrap());

    if let Err(error) = axum::serve(listener, app).await {
        tracing::error!(%error, "HTTP API server stopped with an error");
    }
}

async fn handle_insert(
    State(state): State<SharedState>,
    Json(payload): Json<InsertRequest>,
) -> Json<InsertResponse> {
    tracing::info!(
        "📥 Insert request for ID {} into collection {}",
        payload.vector_id,
        payload.collection
    );
    let mut manager = state.lock().await;

    let part = match manager.route_partition(&payload.collection, payload.vector_id) {
        Ok(p) => p,
        Err(_) => {
            return Json(InsertResponse {
                status: "error: partition not found".into(),
            });
        }
    };

    let meta = payload
        .metadata
        .unwrap_or_else(|| crate::model::VectorMetadata {
            tags: vec![],
            timestamp: 0,
            kv: std::collections::HashMap::new(),
        });

    if part
        .wal
        .append(payload.vector_id, &payload.vector, &meta)
        .is_err()
    {
        return Json(InsertResponse {
            status: "error: failed to write WAL".into(),
        });
    }

    if part
        .segment
        .insert(payload.vector_id, &payload.vector, &meta)
        .is_err()
    {
        return Json(InsertResponse {
            status: "error: failed to write Segment".into(),
        });
    }

    INSERT_COUNT.fetch_add(1, Ordering::Relaxed);

    Json(InsertResponse {
        status: "success".into(),
    })
}

async fn handle_search(
    State(state): State<SharedState>,
    Json(payload): Json<SearchRequest>,
) -> Json<SearchResponse> {
    tracing::info!(
        "🔍 Search request query size {} in collection {}",
        payload.query.len(),
        payload.collection
    );
    let mut manager = state.lock().await;
    let mut all_results = Vec::new();

    let metric = payload
        .metric
        .unwrap_or(crate::storage::search::Metric::Cosine);

    if let Some(col) = manager.collections.get_mut(&payload.collection) {
        for part in &mut col.partitions {
            if let Ok(res) = crate::storage::search::search_segment(
                &mut part.segment,
                &payload.query,
                payload.top_k,
                metric,
                payload.filter.clone(),
            ) {
                for r in res {
                    all_results.push(SearchResultDto {
                        vector_id: r.vector_id,
                        score: r.score,
                        metadata: r.metadata,
                    });
                }
            }
        }
    }

    all_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    all_results.truncate(payload.top_k);

    SEARCH_COUNT.fetch_add(1, Ordering::Relaxed);

    Json(SearchResponse {
        results: all_results,
    })
}

#[derive(Serialize)]
pub struct CollectionDto {
    pub name: String,
}

async fn handle_collections(State(state): State<SharedState>) -> Json<Vec<CollectionDto>> {
    let manager = state.lock().await;
    let collections = manager
        .collections
        .keys()
        .map(|name| CollectionDto { name: name.clone() })
        .collect();
    Json(collections)
}

#[derive(Serialize)]
pub struct PartitionDto {
    pub name: String,
    pub hash_range: (u64, u64),
}

async fn handle_partitions(
    State(state): State<SharedState>,
    Path(collection): Path<String>,
) -> Json<Vec<PartitionDto>> {
    let manager = state.lock().await;
    let mut partitions = Vec::new();
    if let Some(col) = manager.collections.get(&collection) {
        for part in &col.partitions {
            partitions.push(PartitionDto {
                name: part.name.clone(),
                hash_range: part.hash_range,
            });
        }
    }
    Json(partitions)
}

async fn handle_health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn handle_metrics() -> String {
    format!(
        "# HELP resonique_inserts_total Total number of vector inserts\n\
         # TYPE resonique_inserts_total counter\n\
         resonique_inserts_total {}\n\
         # HELP resonique_searches_total Total number of vector searches\n\
         # TYPE resonique_searches_total counter\n\
         resonique_searches_total {}\n",
        INSERT_COUNT.load(Ordering::Relaxed),
        SEARCH_COUNT.load(Ordering::Relaxed)
    )
}
