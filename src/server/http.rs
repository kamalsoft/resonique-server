use axum::{
    Router,
    extract::{DefaultBodyLimit, Json, Path, State, rejection::JsonRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::Mutex;

use crate::server::collection::CollectionManager;

const MAX_REQUEST_BYTES: usize = 1_048_576;
const MAX_VECTOR_DIMENSIONS: usize = 4_096;
const MAX_TOP_K: usize = 1_000;
const MAX_COLLECTION_NAME_LENGTH: usize = 128;

pub static INSERT_COUNT: AtomicU64 = AtomicU64::new(0);
pub static SEARCH_COUNT: AtomicU64 = AtomicU64::new(0);

#[derive(Serialize)]
struct ErrorBody {
    error: ErrorDetails,
}

#[derive(Serialize)]
struct ErrorDetails {
    code: &'static str,
    message: String,
}

struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl ApiError {
    fn bad_request(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code,
            message: message.into(),
        }
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "not_found",
            message: message.into(),
        }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "internal_error",
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorBody {
                error: ErrorDetails {
                    code: self.code,
                    message: self.message,
                },
            }),
        )
            .into_response()
    }
}

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        let status = rejection.status();

        Self {
            status,
            code: if status == StatusCode::PAYLOAD_TOO_LARGE {
                "request_too_large"
            } else {
                "invalid_json"
            },
            message: if status == StatusCode::PAYLOAD_TOO_LARGE {
                format!("request body exceeds the {} byte limit", MAX_REQUEST_BYTES)
            } else {
                "request body must be valid JSON".to_string()
            },
        }
    }
}

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

pub fn build_router(manager: SharedState) -> Router {
    Router::new()
        .route("/health", get(handle_health))
        .route("/metrics", get(handle_metrics))
        .route("/insert", post(handle_insert))
        .route("/search", post(handle_search))
        .route("/collections", get(handle_collections))
        .route(
            "/collections/:collection/partitions",
            get(handle_partitions),
        )
        .layer(DefaultBodyLimit::max(MAX_REQUEST_BYTES))
        .with_state(manager)
}

pub async fn start_http_api(manager: SharedState) {
    let app = build_router(manager);

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

fn validate_collection(collection: &str) -> Result<(), ApiError> {
    if collection.trim().is_empty() {
        return Err(ApiError::bad_request(
            "invalid_collection",
            "collection must not be empty",
        ));
    }

    if collection.len() > MAX_COLLECTION_NAME_LENGTH {
        return Err(ApiError::bad_request(
            "invalid_collection",
            format!(
                "collection name must not exceed {} characters",
                MAX_COLLECTION_NAME_LENGTH
            ),
        ));
    }

    Ok(())
}

fn validate_vector(vector: &[f32], field: &str) -> Result<(), ApiError> {
    if vector.is_empty() {
        return Err(ApiError::bad_request(
            "invalid_vector",
            format!("{field} must not be empty"),
        ));
    }

    if vector.len() > MAX_VECTOR_DIMENSIONS {
        return Err(ApiError::bad_request(
            "vector_too_large",
            format!(
                "{field} must not contain more than {} dimensions",
                MAX_VECTOR_DIMENSIONS
            ),
        ));
    }

    if vector.iter().any(|value| !value.is_finite()) {
        return Err(ApiError::bad_request(
            "invalid_vector",
            format!("{field} must contain only finite numbers"),
        ));
    }

    Ok(())
}

fn validate_top_k(top_k: usize) -> Result<(), ApiError> {
    if top_k == 0 || top_k > MAX_TOP_K {
        return Err(ApiError::bad_request(
            "invalid_top_k",
            format!("top_k must be between 1 and {MAX_TOP_K}"),
        ));
    }

    Ok(())
}

async fn handle_insert(
    State(state): State<SharedState>,
    payload: Result<Json<InsertRequest>, JsonRejection>,
) -> Result<Json<InsertResponse>, ApiError> {
    let Json(payload) = payload.map_err(ApiError::from)?;

    validate_collection(&payload.collection)?;
    validate_vector(&payload.vector, "vector")?;

    tracing::info!(
        "insert request for ID {} into collection {}",
        payload.vector_id,
        payload.collection
    );
    let mut manager = state.lock().await;

    let part = manager
        .route_partition(&payload.collection, payload.vector_id)
        .map_err(|_| ApiError::not_found("collection or partition not found"))?;

    let meta = payload
        .metadata
        .unwrap_or_else(|| crate::model::VectorMetadata {
            tags: vec![],
            timestamp: 0,
            kv: std::collections::HashMap::new(),
        });

    part.wal
        .append(payload.vector_id, &payload.vector, &meta)
        .map_err(|_| ApiError::internal("failed to write WAL"))?;

    part.segment
        .insert(payload.vector_id, &payload.vector, &meta)
        .map_err(|_| ApiError::internal("failed to write segment"))?;

    INSERT_COUNT.fetch_add(1, Ordering::Relaxed);

    Ok(Json(InsertResponse {
        status: "success".into(),
    }))
}

async fn handle_search(
    State(state): State<SharedState>,
    payload: Result<Json<SearchRequest>, JsonRejection>,
) -> Result<Json<SearchResponse>, ApiError> {
    let Json(payload) = payload.map_err(ApiError::from)?;

    validate_collection(&payload.collection)?;
    validate_vector(&payload.query, "query")?;

    validate_top_k(payload.top_k)?;

    let mut manager = state.lock().await;
    let col = manager
        .collections
        .get_mut(&payload.collection)
        .ok_or_else(|| ApiError::not_found("collection not found"))?;

    let metric = payload
        .metric
        .unwrap_or(crate::storage::search::Metric::Cosine);

    let mut all_results = Vec::new();

    for part in &mut col.partitions {
        let results = crate::storage::search::search_segment(
            &mut part.segment,
            &payload.query,
            payload.top_k,
            metric,
            payload.filter.clone(),
        )
        .map_err(|_| ApiError::internal("search failed"))?;

        all_results.extend(results.into_iter().map(|result| SearchResultDto {
            vector_id: result.vector_id,
            score: result.score,
            metadata: result.metadata,
        }));
    }

    all_results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    all_results.truncate(payload.top_k);

    SEARCH_COUNT.fetch_add(1, Ordering::Relaxed);

    Ok(Json(SearchResponse {
        results: all_results,
    }))
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
    pub node_id: String,
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
                node_id: part.node_id.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_collection_name() {
        assert!(validate_collection("default").is_ok());
    }

    #[test]
    fn rejects_empty_collection_name() {
        let error = validate_collection("").unwrap_err();

        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert_eq!(error.code, "invalid_collection");
    }

    #[test]
    fn rejects_collection_name_over_limit() {
        let collection = "a".repeat(MAX_COLLECTION_NAME_LENGTH + 1);

        let error = validate_collection(&collection).unwrap_err();

        assert_eq!(error.code, "invalid_collection");
    }

    #[test]
    fn rejects_empty_vector() {
        let error = validate_vector(&[], "vector").unwrap_err();

        assert_eq!(error.code, "invalid_vector");
    }

    #[test]
    fn rejects_vector_over_dimension_limit() {
        let vector = vec![0.0; MAX_VECTOR_DIMENSIONS + 1];

        let error = validate_vector(&vector, "vector").unwrap_err();

        assert_eq!(error.code, "vector_too_large");
    }

    #[test]
    fn rejects_non_finite_vector_values() {
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let error = validate_vector(&[value], "query").unwrap_err();

            assert_eq!(error.code, "invalid_vector");
        }
    }

    #[test]
    fn accepts_valid_top_k() {
        assert!(validate_top_k(1).is_ok());
        assert!(validate_top_k(MAX_TOP_K).is_ok());
    }

    #[test]
    fn rejects_zero_top_k() {
        let error = validate_top_k(0).unwrap_err();

        assert_eq!(error.code, "invalid_top_k");
    }

    #[test]
    fn rejects_top_k_over_limit() {
        let error = validate_top_k(MAX_TOP_K + 1).unwrap_err();

        assert_eq!(error.code, "invalid_top_k");
    }
}

#[cfg(test)]
mod http_tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use std::time::{SystemTime, UNIX_EPOCH};
    use tower::ServiceExt;

    fn test_app() -> Router {
        let root = std::env::temp_dir().join(format!(
            "resonique_http_test_{}_{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        let engine = crate::storage::StorageEngine::new(&root).unwrap();
        engine.init().unwrap();

        let collections = vec![crate::model::Collection {
            name: "default".to_string(),
            partitions: vec![crate::model::Partition {
                name: "p0".to_string(),
                hash_range: (0, u64::MAX),
                node_id: "node-0".to_string(),
            }],
        }];

        let manager = CollectionManager::new(&engine, collections).unwrap();

        build_router(Arc::new(Mutex::new(manager)))
    }

    async fn error_response(response: axum::response::Response) -> serde_json::Value {
        serde_json::from_slice(
            &axum::body::to_bytes(response.into_body(), MAX_REQUEST_BYTES)
                .await
                .unwrap(),
        )
        .unwrap()
    }

    #[tokio::test]
    async fn malformed_json_returns_consistent_error() {
        let response = test_app()
            .oneshot(
                Request::post("/search")
                    .header("content-type", "application/json")
                    .body(Body::from("{invalid"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = error_response(response).await;
        assert_eq!(body["error"]["code"], "invalid_json");
    }

    #[tokio::test]
    async fn oversized_request_returns_consistent_error() {
        let response = test_app()
            .oneshot(
                Request::post("/insert")
                    .header("content-type", "application/json")
                    .body(Body::from(vec![b'x'; MAX_REQUEST_BYTES + 1]))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);

        let body = error_response(response).await;
        assert_eq!(body["error"]["code"], "request_too_large");
    }

    #[tokio::test]
    async fn invalid_vector_returns_consistent_error() {
        let response = test_app()
            .oneshot(
                Request::post("/search")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"collection":"default","query":[],"top_k":1}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = error_response(response).await;
        assert_eq!(body["error"]["code"], "invalid_vector");
    }

    #[tokio::test]
    async fn invalid_top_k_returns_consistent_error() {
        let response = test_app()
            .oneshot(
                Request::post("/search")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"collection":"default","query":[1.0],"top_k":0}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = error_response(response).await;
        assert_eq!(body["error"]["code"], "invalid_top_k");
    }

    #[tokio::test]
    async fn unknown_collection_returns_consistent_error() {
        let response = test_app()
            .oneshot(
                Request::post("/search")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"collection":"missing","query":[1.0],"top_k":1}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = error_response(response).await;
        assert_eq!(body["error"]["code"], "not_found");
    }
}
