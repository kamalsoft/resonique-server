use crate::model::VectorMetadata;
use crate::server::collection::CollectionManager;
use crate::storage::search::{Metric, QueryFilter};
use anyhow::Result;
use serde::Deserialize;
use serde_json::{Value, json};
use std::cmp::Ordering;
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;

type SharedManager = Arc<Mutex<CollectionManager>>;

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: Option<String>,
    id: Option<Value>,
    method: Option<String>,
    #[serde(default)]
    params: Value,
}

pub async fn start(manager: SharedManager) -> Result<()> {
    eprintln!("🔌 MCP transport starting (stdio)...");

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = BufReader::new(stdin).lines();
    let mut stdout = stdout;

    while let Some(line) = reader.next_line().await? {
        let response = match serde_json::from_str::<JsonRpcRequest>(&line) {
            Ok(request) => dispatch(request, &manager).await,
            Err(error) => error_response(Value::Null, -32700, format!("Invalid JSON: {error}")),
        };

        let encoded = serde_json::to_vec(&response)?;
        stdout.write_all(&encoded).await?;
        stdout.write_all(b"\n").await?;
        stdout.flush().await?;
    }

    Ok(())
}

async fn dispatch(request: JsonRpcRequest, manager: &SharedManager) -> Value {
    let id = request.id.clone().unwrap_or(Value::Null);

    if request.jsonrpc.as_deref() != Some("2.0") {
        return error_response(id, -32600, "jsonrpc must be \"2.0\"".to_string());
    }

    let Some(method) = request.method.as_deref() else {
        return error_response(id, -32600, "Missing method".to_string());
    };

    match method {
        "ping" => success_response(id, json!({ "pong": true })),

        "initialize" => success_response(
            id,
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "resonique-server",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }),
        ),

        "tools/list" | "list_tools" => success_response(
            id,
            json!({
                "tools": tool_definitions()
            }),
        ),

        "tools/call" | "call_tool" => {
            let Some(name) = request.params.get("name").and_then(Value::as_str) else {
                return error_response(id, -32602, "tools/call requires params.name".to_string());
            };

            let arguments = request
                .params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));

            match call_tool(name, arguments, manager).await {
                Ok(value) => success_response(id, value),
                Err(error) => tool_error_response(id, error.to_string()),
            }
        }

        _ => error_response(id, -32601, format!("Method not found: {method}")),
    }
}

fn tool_definitions() -> Value {
    json!([
        {
            "name": "insert_vector",
            "description": "Insert a vector and optional metadata into a collection",
            "inputSchema": {
                "type": "object",
                "required": ["collection", "vector_id", "vector"],
                "properties": {
                    "collection": { "type": "string" },
                    "vector_id": { "type": "integer", "minimum": 0 },
                    "vector": {
                        "type": "array",
                        "items": { "type": "number" },
                        "minItems": 1
                    },
                    "metadata": {
                        "type": "object",
                        "properties": {
                            "tags": {
                                "type": "array",
                                "items": { "type": "string" }
                            },
                            "timestamp": { "type": "integer", "minimum": 0 },
                            "kv": {
                                "type": "object",
                                "additionalProperties": { "type": "string" }
                            }
                        }
                    }
                }
            }
        },
        {
            "name": "search_vectors",
            "description": "Search vectors using cosine similarity or L2 distance",
            "inputSchema": {
                "type": "object",
                "required": ["collection", "query", "top_k"],
                "properties": {
                    "collection": { "type": "string" },
                    "query": {
                        "type": "array",
                        "items": { "type": "number" },
                        "minItems": 1
                    },
                    "top_k": { "type": "integer", "minimum": 1 }
                }
            }
        },
    ])
}

async fn call_tool(name: &str, arguments: Value, manager: &SharedManager) -> Result<Value> {
    match name {
        "insert_vector" => insert_vector(arguments, manager).await,
        "search_vectors" => search_vectors(arguments, manager).await,
        "list_collections" => list_collections(manager).await,
        "list_partitions" => list_partitions(arguments, manager).await,
        "health_mcp" => health_mcp(manager).await,
        _ => anyhow::bail!("Unknown tool: {name}"),
    }
}

async fn insert_vector(arguments: Value, manager: &SharedManager) -> Result<Value> {
    let collection = required_string(&arguments, "collection")?;
    let vector_id = required_u64(&arguments, "vector_id")?;
    let vector = required_vector(&arguments, "vector")?;

    if vector.is_empty() {
        anyhow::bail!("vector must not be empty");
    }

    let metadata = match arguments.get("metadata") {
        Some(value) => serde_json::from_value::<VectorMetadata>(value.clone())
            .map_err(|error| anyhow::anyhow!("invalid metadata: {error}"))?,
        None => VectorMetadata::default(),
    };

    let mut state = manager.lock().await;
    let partition = state.route_partition(&collection, vector_id)?;

    partition.wal.append(vector_id, &vector, &metadata)?;
    partition.segment.insert(vector_id, &vector, &metadata)?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!("Vector {vector_id} inserted into collection {collection}")
        }],
        "structuredContent": {
            "status": "success",
            "collection": collection,
            "vector_id": vector_id,
            "metadata": metadata
        },
        "isError": false
    }))
}

async fn search_vectors(arguments: Value, manager: &SharedManager) -> Result<Value> {
    let collection = required_string(&arguments, "collection")?;
    let query = required_vector(&arguments, "query")?;
    let top_k = required_u64(&arguments, "top_k")?;

    if query.is_empty() {
        anyhow::bail!("query must not be empty");
    }

    if top_k == 0 {
        anyhow::bail!("top_k must be greater than zero");
    }

    let top_k = usize::try_from(top_k).map_err(|_| anyhow::anyhow!("top_k is too large"))?;

    let metric = match arguments.get("metric") {
        Some(value) => serde_json::from_value::<Metric>(value.clone())
            .map_err(|error| anyhow::anyhow!("invalid metric: {error}"))?,
        None => Metric::Cosine,
    };

    let filter = match arguments.get("filter") {
        Some(value) => Some(
            serde_json::from_value::<QueryFilter>(value.clone())
                .map_err(|error| anyhow::anyhow!("invalid filter: {error}"))?,
        ),
        None => None,
    };

    let mut state = manager.lock().await;
    let collection_state = state
        .collections
        .get_mut(&collection)
        .ok_or_else(|| anyhow::anyhow!("Collection not found: {collection}"))?;

    let mut results = Vec::new();

    for partition in &mut collection_state.partitions {
        let partition_results = crate::storage::search::search_segment(
            &mut partition.segment,
            &query,
            top_k,
            metric,
            filter.clone(),
        )?;

        results.extend(partition_results);
    }

    match metric {
        Metric::Cosine => {
            results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal))
        }
        Metric::L2 => {
            results.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(Ordering::Equal))
        }
    }

    results.truncate(top_k);

    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&results)?
        }],
        "structuredContent": {
            "collection": collection,
            "results": results
        },
        "isError": false
    }))
}

async fn list_collections(manager: &SharedManager) -> Result<Value> {
    let state = manager.lock().await;

    let collections: Vec<Value> = state
        .collections
        .keys()
        .map(|name| json!({ "name": name }))
        .collect();

    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&collections)?
        }],
        "structuredContent": {
            "collections": collections
        },
        "isError": false
    }))
}

async fn list_partitions(arguments: Value, manager: &SharedManager) -> Result<Value> {
    let collection = required_string(&arguments, "collection")?;
    let state = manager.lock().await;

    let collection_state = state
        .collections
        .get(&collection)
        .ok_or_else(|| anyhow::anyhow!("Collection not found: {collection}"))?;

    let partitions: Vec<Value> = collection_state
        .partitions
        .iter()
        .map(|partition| {
            json!({
                "name": partition.name,
                "hash_range": partition.hash_range
            })
        })
        .collect();

    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&partitions)?
        }],
        "structuredContent": {
            "collection": collection,
            "partitions": partitions
        },
        "isError": false
    }))
}

async fn health_mcp(manager: &SharedManager) -> Result<Value> {
    let state = manager.lock().await;

    let partition_count: usize = state
        .collections
        .values()
        .map(|collection| collection.partitions.len())
        .sum();

    Ok(json!({
        "content": [{
            "type": "text",
            "text": "MCP transport and collection manager are healthy"
        }],
        "structuredContent": {
            "status": "ok",
            "transport": "stdio",
            "collections": state.collections.len(),
            "partitions": partition_count
        },
        "isError": false
    }))
}

fn required_string(arguments: &Value, field: &str) -> Result<String> {
    let value = arguments
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("{field} must be a non-empty string"))?;

    if value.trim().is_empty() {
        anyhow::bail!("{field} must be a non-empty string");
    }

    Ok(value.to_owned())
}

fn required_u64(arguments: &Value, field: &str) -> Result<u64> {
    arguments
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| anyhow::anyhow!("{field} must be a non-negative integer"))
}

fn required_vector(arguments: &Value, field: &str) -> Result<Vec<f32>> {
    let value = arguments
        .get(field)
        .ok_or_else(|| anyhow::anyhow!("missing field: {field}"))?;

    let vector = serde_json::from_value::<Vec<f32>>(value.clone())
        .map_err(|error| anyhow::anyhow!("{field} must be an array of numbers: {error}"))?;

    if vector.iter().any(|value| !value.is_finite()) {
        anyhow::bail!("{field} contains a non-finite value");
    }

    Ok(vector)
}

fn success_response(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

fn error_response(id: Value, code: i32, message: String) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
}

fn tool_error_response(id: Value, message: String) -> Value {
    success_response(
        id,
        json!({
            "content": [{
                "type": "text",
                "text": message
            }],
            "isError": true
        }),
    )
}
