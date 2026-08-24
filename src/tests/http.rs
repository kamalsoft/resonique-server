use crate::server::collection::CollectionManager;
use crate::server::http::*;
use axum::{Router, http::StatusCode};
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex as StdMutex;

    static ENV_LOCK: StdMutex<()> = StdMutex::new(());

    #[test]
    fn uses_default_http_address() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe { std::env::remove_var("RESONIQUE_HTTP_ADDR") };

        assert_eq!(http_bind_addr(), "127.0.0.1:3000".parse().unwrap());
    }

    #[test]
    fn uses_configured_http_address() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe { std::env::set_var("RESONIQUE_HTTP_ADDR", "0.0.0.0:8080") };

        assert_eq!(http_bind_addr(), "0.0.0.0:8080".parse().unwrap());

        unsafe { std::env::remove_var("RESONIQUE_HTTP_ADDR") };
    }

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

    #[tokio::test]
    async fn excessive_top_k_returns_consistent_error() {
        let response = test_app()
            .oneshot(
                Request::post("/search")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{"collection":"default","query":[1.0],"top_k":{}}}"#,
                        MAX_TOP_K + 1
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = error_response(response).await;
        assert_eq!(body["error"]["code"], "invalid_top_k");
    }

    #[tokio::test]
    async fn health_endpoint_returns_ok() {
        let response = test_app()
            .oneshot(Request::get("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = error_response(response).await;
        assert_eq!(body["status"], "ok");
    }

    #[tokio::test]
    async fn collections_endpoint_lists_collections() {
        let response = test_app()
            .oneshot(Request::get("/collections").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = error_response(response).await;
        assert_eq!(body[0]["name"], "default");
    }

    #[tokio::test]
    async fn partitions_endpoint_lists_partitions() {
        let response = test_app()
            .oneshot(
                Request::get("/collections/default/partitions")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = error_response(response).await;

        assert_eq!(body[0]["name"], "p0");
        assert_eq!(body[0]["node_id"], "node-0");
    }

    #[tokio::test]
    async fn missing_partitions_return_empty_array() {
        let response = test_app()
            .oneshot(
                Request::get("/collections/missing/partitions")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(error_response(response).await, serde_json::json!([]));
    }

    #[tokio::test]
    async fn insert_and_search_endpoints_work() {
        let app = test_app();

        let response = app
            .clone()
            .oneshot(
                Request::post("/insert")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{
                            "collection":"default",
                            "vector_id":42,
                            "vector":[1.0,0.0]
                        }"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(error_response(response).await["status"], "success");

        let response = app
            .oneshot(
                Request::post("/search")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{
                            "collection":"default",
                            "query":[1.0,0.0],
                            "top_k":1
                        }"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = error_response(response).await;

        assert_eq!(body["results"][0]["vector_id"], 42);
    }

    #[tokio::test]
    async fn insert_rejects_unknown_collection() {
        let response = test_app()
            .oneshot(
                Request::post("/insert")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{
                            "collection":"missing",
                            "vector_id":1,
                            "vector":[1.0]
                        }"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(error_response(response).await["error"]["code"], "not_found");
    }

    #[tokio::test]
    async fn metrics_endpoint_returns_prometheus_metrics() {
        let response = test_app()
            .oneshot(Request::get("/metrics").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = String::from_utf8(
            axum::body::to_bytes(response.into_body(), MAX_REQUEST_BYTES)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();

        assert!(body.contains("resonique_inserts_total"));
        assert!(body.contains("resonique_searches_total"));
    }

    #[tokio::test]
    async fn invalid_collection_is_rejected_by_handlers() {
        let response = test_app()
            .oneshot(
                Request::post("/insert")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{
                            "collection":"",
                            "vector_id":1,
                            "vector":[1.0]
                        }"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            error_response(response).await["error"]["code"],
            "invalid_collection"
        );
    }

    #[tokio::test]
    async fn search_rejects_unknown_collection() {
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
        assert_eq!(error_response(response).await["error"]["code"], "not_found");
    }

    #[tokio::test]
    async fn search_rejects_malformed_json() {
        let response = test_app()
            .oneshot(
                Request::post("/search")
                    .header("content-type", "application/json")
                    .body(Body::from("{invalid-json"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            error_response(response).await["error"]["code"],
            "invalid_json"
        );
    }

    #[tokio::test]
    async fn insert_rejects_malformed_json() {
        let response = test_app()
            .oneshot(
                Request::post("/insert")
                    .header("content-type", "application/json")
                    .body(Body::from("{invalid-json"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            error_response(response).await["error"]["code"],
            "invalid_json"
        );
    }
}
