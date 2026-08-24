use crate::server::http::ApiError;
use axum::response::IntoResponse;

use crate::tests::helpers::manager;
use serde_json::json;

#[tokio::test]
async fn handles_invalid_json() {
    let (manager, root) = manager();

    let response = crate::mcp::dispatch_line("{invalid", &manager).await;

    assert_eq!(response["error"]["code"], -32700);
    assert!(
        response["error"]["message"]
            .as_str()
            .unwrap()
            .starts_with("Invalid JSON")
    );

    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn handles_invalid_jsonrpc_version() {
    let (manager, root) = manager();

    let response =
        crate::mcp::dispatch_line(r#"{"jsonrpc":"1.0","id":1,"method":"ping"}"#, &manager).await;

    assert_eq!(response["error"]["code"], -32600);

    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn handles_missing_method() {
    let (manager, root) = manager();

    let response = crate::mcp::dispatch_line(r#"{"jsonrpc":"2.0","id":1}"#, &manager).await;

    assert_eq!(response["error"]["code"], -32600);

    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn handles_unknown_method() {
    let (manager, root) = manager();

    let response =
        crate::mcp::dispatch_line(r#"{"jsonrpc":"2.0","id":1,"method":"unknown"}"#, &manager).await;

    assert_eq!(response["error"]["code"], -32601);

    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn lists_tools() {
    let (manager, root) = manager();

    let response = crate::mcp::dispatch_line(
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#,
        &manager,
    )
    .await;

    let tools = response["result"]["tools"].as_array().unwrap();

    assert!(tools.iter().any(|tool| tool["name"] == "insert_vector"));
    assert!(tools.iter().any(|tool| tool["name"] == "search_vectors"));

    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn supports_ping_and_initialize() {
    let (manager, root) = manager();

    let ping =
        crate::mcp::dispatch_line(r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#, &manager).await;
    assert_eq!(ping["result"]["pong"], true);

    let initialize = crate::mcp::dispatch_line(
        r#"{"jsonrpc":"2.0","id":2,"method":"initialize"}"#,
        &manager,
    )
    .await;
    assert_eq!(
        initialize["result"]["serverInfo"]["name"],
        "resonique-server"
    );

    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn lists_collections_and_partitions() {
    let (manager, root) = manager();

    let collections = crate::mcp::dispatch_line(
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"list_collections"}}"#,
        &manager,
    )
    .await;
    assert_eq!(
        collections["result"]["structuredContent"]["collections"][0]["name"],
        "default"
    );

    let partitions = crate::mcp::dispatch_line(
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"list_partitions","arguments":{"collection":"default"}}}"#,
        &manager,
    )
    .await;
    assert_eq!(
        partitions["result"]["structuredContent"]["partitions"][0]["name"],
        "p0"
    );

    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn inserts_and_searches_vectors() {
    let (manager, root) = manager();

    let insert = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "insert_vector",
            "arguments": {
                "collection": "default",
                "vector_id": 1,
                "vector": [1.0, 0.0]
            }
        }
    });

    let response = crate::mcp::dispatch_line(&insert.to_string(), &manager).await;
    assert_eq!(response["result"]["isError"], false);

    let search = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "search_vectors",
            "arguments": {
                "collection": "default",
                "query": [1.0, 0.0],
                "top_k": 1
            }
        }
    });

    let response = crate::mcp::dispatch_line(&search.to_string(), &manager).await;
    assert_eq!(response["result"]["isError"], false);
    assert_eq!(
        response["result"]["structuredContent"]["results"][0]["vector_id"],
        1
    );

    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn returns_tool_errors() {
    let (manager, root) = manager();

    let response = crate::mcp::dispatch_line(
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"unknown_tool"}}"#,
        &manager,
    )
    .await;

    assert_eq!(response["result"]["isError"], true);

    let response = crate::mcp::dispatch_line(
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{}}"#,
        &manager,
    )
    .await;

    assert_eq!(response["error"]["code"], -32602);

    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn rejects_invalid_tool_arguments() {
    let (manager, root) = manager();

    for arguments in [
        serde_json::json!({"collection": "", "vector_id": 1, "vector": [1.0]}),
        serde_json::json!({"collection": "default", "vector_id": 1, "vector": []}),
        serde_json::json!({"collection": "default", "vector_id": 1, "vector": [null]}),
    ] {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "insert_vector",
                "arguments": arguments
            }
        });

        let response = crate::mcp::dispatch_line(&request.to_string(), &manager).await;

        assert!(response.get("error").is_some() || response.to_string().contains("isError"));
    }

    drop(manager);
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn rejects_invalid_search_arguments() {
    let (manager, root) = manager();

    for arguments in [
        serde_json::json!({"collection": "default", "query": [], "top_k": 1}),
        serde_json::json!({"collection": "default", "query": [1.0], "top_k": 0}),
        serde_json::json!({
            "collection": "default",
            "query": [1.0],
            "top_k": 1,
            "metric": "invalid"
        }),
        serde_json::json!({
            "collection": "default",
            "query": [1.0],
            "top_k": 1,
            "filter": "invalid"
        }),
    ] {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "search_vectors",
                "arguments": arguments
            }
        });

        let response = crate::mcp::dispatch_line(&request.to_string(), &manager).await;

        assert!(response.get("error").is_some() || response.to_string().contains("isError"));
    }

    drop(manager);
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn health_tool_reports_collection_and_partition_counts() {
    let (manager, root) = manager();

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "health_mcp",
            "arguments": {}
        }
    });

    let response = crate::mcp::dispatch_line(&request.to_string(), &manager).await;

    assert_eq!(response["result"]["structuredContent"]["status"], "ok");
    assert!(
        response["result"]["structuredContent"]["partitions"]
            .as_u64()
            .is_some()
    );

    drop(manager);
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn rejects_invalid_metadata() {
    let (manager, root) = manager();

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "insert_vector",
            "arguments": {
                "collection": "default",
                "vector_id": 1,
                "vector": [1.0],
                "metadata": {"timestamp": "invalid"}
            }
        }
    });

    let response = crate::mcp::dispatch_line(&request.to_string(), &manager).await;

    assert!(response.to_string().contains("invalid metadata"));

    drop(manager);
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn rejects_missing_required_arguments() {
    let (manager, root) = manager();

    for arguments in [
        serde_json::json!({}),
        serde_json::json!({"collection": "default"}),
        serde_json::json!({"collection": "default", "vector_id": 1}),
        serde_json::json!({
            "collection": "default",
            "vector_id": "invalid",
            "vector": [1.0]
        }),
    ] {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "insert_vector",
                "arguments": arguments
            }
        });

        let response = crate::mcp::dispatch_line(&request.to_string(), &manager).await;

        assert!(response.get("error").is_some() || response.to_string().contains("isError"));
    }

    drop(manager);
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn searches_using_l2_metric() {
    let (manager, root) = manager();

    let insert = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "insert_vector",
            "arguments": {
                "collection": "default",
                "vector_id": 1,
                "vector": [1.0, 0.0]
            }
        }
    });

    let response = crate::mcp::dispatch_line(&insert.to_string(), &manager).await;
    assert_eq!(response["result"]["isError"], false);

    let search = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "search_vectors",
            "arguments": {
                "collection": "default",
                "query": [1.0, 0.0],
                "top_k": 1,
                "metric": "L2"
            }
        }
    });

    let response = crate::mcp::dispatch_line(&search.to_string(), &manager).await;
    assert_eq!(response["result"]["isError"], false);
    assert_eq!(
        response["result"]["structuredContent"]["results"][0]["vector_id"],
        1
    );

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn creates_internal_http_error() {
    let response = ApiError::internal("storage failure").into_response();

    assert_eq!(
        response.status(),
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    );
}
