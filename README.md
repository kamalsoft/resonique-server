# Resonique Server

Resonique Server is a Rust-based vector storage and similarity-search service. It provides durable write-ahead logging, segment storage, metadata filtering, partition routing, HTTP APIs, and an MCP stdio transport.

## Current capabilities

- Rust 2024 with Tokio
- Axum HTTP server
- MCP stdio transport with JSON-RPC ping handling
- Collection and range-based partition routing
- Append-only JSON-lines WAL
- Segment-backed vector and metadata storage
- Cosine similarity and L2 distance search
- Tag and key-value metadata filtering
- Secondary-index implementation
- Health and metrics endpoints
- WAL replay during startup
- Unit, integration, performance, and security tests

## Project layout

```text
src/
├── main.rs
├── lib.rs
├── model/
├── server/
│   ├── mod.rs
│   ├── collection.rs
│   └── http.rs
├── storage/
│   ├── mod.rs
│   ├── index.rs
│   ├── search.rs
│   ├── segment.rs
│   └── wal.rs
├── mcp/
└── tests/
```

## Requirements

- Rust stable
- Cargo

## Run

```bash
cargo run
```

The server reads `config.json`, initializes the storage directory, starts the MCP stdio transport, and starts the HTTP API on `127.0.0.1:3000`.

## Configuration

```json
{
  "storage_root": "./data",
  "collections": [
    {
      "name": "default",
      "partitions": [
        {
          "name": "p0",
          "hash_range": [0, 18446744073709551615]
        }
      ]
    }
  ]
}
```

## HTTP API

### Health

```bash
curl http://127.0.0.1:3000/health
```

### Metrics

```bash
curl http://127.0.0.1:3000/metrics
```

### Insert

```bash
curl -X POST http://127.0.0.1:3000/insert \
  -H 'content-type: application/json' \
  -d '{
    "collection": "default",
    "vector_id": 1,
    "vector": [1.0, 0.0],
    "metadata": {
      "tags": ["example"],
      "timestamp": 1700000000,
      "kv": {"source": "demo"}
    }
  }'
```

### Search

```bash
curl -X POST http://127.0.0.1:3000/search \
  -H 'content-type: application/json' \
  -d '{
    "collection": "default",
    "query": [1.0, 0.0],
    "top_k": 10,
    "metric": "Cosine",
    "filter": {
      "tag": "example",
      "metadata_key": null,
      "metadata_val": null
    }
  }'
```

### Collections and partitions

```bash
curl http://127.0.0.1:3000/collections
curl http://127.0.0.1:3000/collections/default/partitions
```

## API Reference

The HTTP API base URL is:

```text
http://127.0.0.1:3000
```

All JSON requests should include:

```http
Content-Type: application/json
```

### Endpoint summary

| Method | Endpoint | Purpose |
|---|---|---|
| `GET` | `/health` | Check HTTP process availability |
| `GET` | `/metrics` | Read process-local operation counters |
| `GET` | `/collections` | List configured collections |
| `GET` | `/collections/:collection/partitions` | List collection partitions |
| `POST` | `/insert` | Persist a vector and metadata |
| `POST` | `/search` | Search vectors across collection partitions |

### Health-check interpretation

A successful response from `/health` confirms that:

- The HTTP listener is running.
- The process is accepting requests.
- The Axum router is available.

It does not independently verify every segment, WAL, index, or partition file.

Use `curl -i` when diagnosing status codes and headers:

```bash
curl -i http://127.0.0.1:3000/health
```

Expected response:

```http
HTTP/1.1 200 OK
content-type: application/json

{"status":"ok"}
```

### Metrics interpretation

The `/metrics` counters are process-local:

- `resonique_inserts_total` counts successful insert operations.
- `resonique_searches_total` counts completed search operations.
- Counters reset when the process restarts.
- Metrics are returned as text in Prometheus-compatible exposition format.

Example:

```bash
curl -s http://127.0.0.1:3000/metrics
```

### Search result fields

A search result contains:

| Field | Description |
|---|---|
| `vector_id` | Identifier supplied during insertion |
| `score` | Cosine similarity or L2 distance |
| `metadata` | Tags, timestamp, and key-value metadata when returned by the active build |

For cosine similarity, larger scores represent closer matches. For L2 distance, smaller scores represent closer matches.

## API Smoke Test

Start the service in one terminal:

```bash
cargo run
```

Run the following commands from a second terminal:

```bash
BASE_URL=http://127.0.0.1:3000

curl --fail-with-body -sS "$BASE_URL/health"
printf '\n'

curl --fail-with-body -sS "$BASE_URL/collections"
printf '\n'

curl --fail-with-body -sS "$BASE_URL/collections/default/partitions"
printf '\n'

curl --fail-with-body -sS -X POST "$BASE_URL/insert" \
  -H 'content-type: application/json' \
  -d '{
    "collection": "default",
    "vector_id": 100,
    "vector": [1.0, 0.0],
    "metadata": {
      "tags": ["smoke-test"],
      "timestamp": 1700000000,
      "kv": {
        "source": "readme"
      }
    }
  }'
printf '\n'

curl --fail-with-body -sS -X POST "$BASE_URL/search" \
  -H 'content-type: application/json' \
  -d '{
    "collection": "default",
    "query": [1.0, 0.0],
    "top_k": 5,
    "metric": "Cosine",
    "filter": {
      "metadata_key": "source",
      "metadata_val": "readme"
    }
  }'
printf '\n'

curl --fail-with-body -sS "$BASE_URL/metrics"
```

`--fail-with-body` makes `curl` return a failure status for HTTP errors while preserving the response body for diagnosis.

## Metadata Filtering

Metadata is attached to each vector during insertion.

Tag filtering:

```json
{
  "filter": {
    "tag": "smoke-test"
  }
}
```

Key-value filtering:

```json
{
  "filter": {
    "metadata_key": "source",
    "metadata_val": "readme"
  }
}
```

The key-value filter matches the requested value exactly. If the requested metadata key or value is not present, the vector is excluded from the results.

## Persistence and Recovery

The server stores one segment file and one WAL file for each configured collection partition:

```text
<storage_root>/<collection>_<partition>.segment
<storage_root>/<collection>_<partition>.wal
```

The insert sequence is:

1. Route the vector ID to a partition.
2. Append the vector and metadata to the WAL.
3. Write the vector and metadata to the segment.
4. Update the in-memory segment index.
5. Return the insert response.

During startup, the server opens the segment and WAL files and replays WAL records to reconstruct the in-memory index.

Back up data before changing storage formats or deleting files:

```bash
cp -R data "data.backup.$(date +%Y%m%d%H%M%S)"
```

To intentionally reset development data:

```bash
rm -rf data
```

This permanently removes locally stored vectors and metadata.

## MCP Transport and Testing

MCP uses JSON-RPC over standard input and standard output. It is separate from the HTTP API:

```text
MCP  → stdin/stdout
HTTP → http://127.0.0.1:3000
```

The current MCP implementation supports:

| Method or tool | Purpose |
|---|---|
| `ping` | Basic transport check |
| `initialize` | MCP protocol handshake |
| `tools/list` | List available tools |
| `tools/call` | Invoke a tool |
| `insert_vector` | Insert a vector and metadata |
| `search_vectors` | Search vectors and return metadata |
| `list_collections` | List configured collections |
| `list_partitions` | List partitions for a collection |
| `health_mcp` | Check MCP and collection-manager health |

### MCP protocol smoke test

Run this from the repository root:

```bash
cargo build

cargo run --quiet >mcp-responses.jsonl 2>mcp-diagnostics.log <<'EOF'
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"health_mcp","arguments":{}}}
EOF
```

Inspect protocol responses and diagnostics separately:

```bash
cat mcp-responses.jsonl
cat mcp-diagnostics.log
```

JSON-RPC responses are written to stdout. Startup and diagnostic messages are written to stderr.

### Initialize and list tools

```bash
printf '%s\n' \
'{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' \
'{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' \
| cargo run --quiet 2>/dev/null
```

The initialization response contains the protocol version, server information, and tool capability.

### List collections and partitions

```bash
printf '%s\n' \
'{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"list_collections","arguments":{}}}' \
'{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"list_partitions","arguments":{"collection":"default"}}}' \
| cargo run --quiet 2>/dev/null
```

### Insert a vector

```bash
printf '%s\n' \
'{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"insert_vector","arguments":{"collection":"default","vector_id":9001,"vector":[1.0,0.0],"metadata":{"tags":["demo"],"timestamp":1700000000,"kv":{"source":"mcp-test","category":"example"}}}}}' \
| cargo run --quiet 2>/dev/null
```

A successful result includes readable metadata:

```json
{
  "status": "success",
  "collection": "default",
  "vector_id": 9001,
  "metadata": {
    "tags": ["demo"],
    "timestamp": 1700000000,
    "kv": {
      "source": "mcp-test",
      "category": "example"
    }
  }
}
```

### Search vectors

```bash
printf '%s\n' \
'{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"search_vectors","arguments":{"collection":"default","query":[1.0,0.0],"top_k":5,"metric":"Cosine","filter":{"metadata_key":"source","metadata_val":"mcp-test"}}}}' \
| cargo run --quiet 2>/dev/null
```

A matching result contains:

```json
{
  "vector_id": 9001,
  "score": 1.0,
  "metadata": {
    "tags": ["demo"],
    "timestamp": 1700000000,
    "kv": {
      "source": "mcp-test",
      "category": "example"
    }
  }
}
```

For cosine similarity, larger scores are better. For L2 distance, smaller scores are better.

### MCP health check

```bash
printf '%s\n' \
'{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"health_mcp","arguments":{}}}' \
| cargo run --quiet 2>/dev/null
```

A healthy response includes:

```json
{
  "status": "ok",
  "transport": "stdio"
}
```

### MCP error handling

Malformed JSON:

```bash
printf '%s\n' '{invalid-json}' \
| cargo run --quiet 2>/dev/null
```

Unknown method:

```bash
printf '%s\n' \
'{"jsonrpc":"2.0","id":1,"method":"unknown","params":{}}' \
| cargo run --quiet 2>/dev/null
```

Unknown tool:

```bash
printf '%s\n' \
'{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"missing_tool","arguments":{}}}' \
| cargo run --quiet 2>/dev/null
```

Expected JSON-RPC error codes:

| Code | Meaning |
|---:|---|
| `-32700` | Invalid JSON |
| `-32600` | Invalid JSON-RPC request |
| `-32601` | Method or tool not found |
| `-32602` | Invalid method parameters |

Tool execution failures are returned in a JSON-RPC success response with `"isError": true`.

### MCP validation cases

The following inputs should be rejected:

- Missing or empty `collection`.
- Missing `vector_id`.
- Missing, empty, or non-numeric `vector` or `query`.
- Non-finite vector values.
- `top_k` equal to zero.
- Invalid metric values.
- Unknown collections.
- Invalid metadata structures.
- Unknown MCP tools.

### MCP verification

```bash
cargo fmt -- --check
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo build --release
```

MCP is not sent to the HTTP port. Use stdin/stdout for MCP and `curl` for HTTP:

```bash
curl -i http://127.0.0.1:3000/health
```

## Troubleshooting

### Port `3000` is already in use

```bash
lsof -nP -iTCP:3000 -sTCP:LISTEN
```

After identifying the process:

```bash
kill <PID>
```

Use `kill -9 <PID>` only if the process does not stop normally.

### The server starts but health requests fail

Confirm that the listener is active:

```bash
lsof -nP -iTCP:3000 -sTCP:LISTEN
```

Then test the loopback address explicitly:

```bash
curl -v http://127.0.0.1:3000/health
```

### Startup fails while reading WAL data

Preserve the data directory first:

```bash
cp -R data data.backup
```

Inspect WAL files:

```bash
find data -type f -name '*.wal' -print
```

A malformed or incompatible WAL record can prevent recovery. Do not delete the WAL until the data-loss impact is understood.

### Insert returns an error status

Check:

- The collection name matches `config.json`.
- The vector ID belongs to a configured partition range.
- The storage directory is writable.
- The segment and WAL files are accessible.
- The request body is valid JSON.
- The vector is not empty.

### Search returns no results

Check:

- The collection exists.
- `top_k` is greater than zero.
- The query vector has the expected dimension.
- The query is close to the inserted vector.
- Metadata filters match exactly.
- The insert operation completed successfully.

## Development and Release Checklist

Run the complete verification sequence before committing:

```bash
cargo fmt
cargo fmt -- --check
cargo check --all-targets --all-features
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release
cargo audit
```

Run tests with logs visible:

```bash
cargo test --all-targets --all-features -- --nocapture
```

Check the release binary:

```bash
ls -lh target/release/resonique-server
```

For API changes, also perform the API smoke test and update the relevant files under `docs/`.

## Documentation Map

| Document | Scope |
|---|---|
| `README.md` | Setup, API usage, operations, and troubleshooting |
| `docs/architecture.md` | Component architecture and runtime lifecycle |
| `docs/hld.md` | High-level design and request flows |
| `docs/lld.md` | Module, struct, and storage-format details |
| `docs/topology.md` | Single-node deployment and future topology |
| `docs/visuals/architecture-diagram.md` | Component diagram |
| `docs/visuals/data-flow-diagram.md` | Insert and search flow |
| `docs/visuals/module-dependencies.md` | Rust module dependencies |

## Implemented Features

- Rust server built and managed with Cargo.
- HTTP API on the configured local address.
- Health-check endpoint.
- MCP stdio transport.
- Configurable storage root.
- Collections and partitions.
- Segment-based persistent storage.
- Write-ahead logging and replay.
- Secondary indexing.
- Search operations.
- Path-traversal protection.
- Unit, integration, security, and performance tests.
- Architecture, data-flow, topology, and module documentation.
-  Upgrade and migration runbooks
- Request limits and consistent errors 

## Roadmap

### High Priority

| Area | Item | Status |
|---|---|---|
| Developer Experience | CI checks for tests, Clippy, audit, and dependencies | TODO |
| Documentation | Security, backup, and recovery runbooks | TODO |
| MCP | Production protocol validation and interoperability tests | TODO |
| Observability | Readiness and liveness endpoints | TODO |
| Scalability | Concurrent load and saturation testing | TODO |
| Security | Authentication and authorization | TODO |
| Storage | Backup and restore procedures | TODO |
| Storage | Corruption and crash-injection testing | TODO |

### Medium Priority

| Area | Item | Status |
|---|---|---|
| APIs | API schema and versioning | TODO |
| APIs | Pagination and rate limiting | TODO |
| Developer Experience | Reproducible benchmark and test-data tooling | TODO |
| Documentation | API reference and deployment guide | TODO |
| gRPC | gRPC service, protobuf contract, and client tests | TODO |
| Metadata | Persistent metadata catalog and migrations | TODO |
| Observability | Distributed tracing and request IDs | TODO |
| Observability | Expanded Prometheus metrics | TODO |
| Storage | Segment compaction and lifecycle management | TODO |

### Low Priority

| Area | Item | Status |
|---|---|---|
| Documentation | Upgrade and migration runbooks | TODO |
| Scalability | Horizontal scaling and partition distribution | TODO |
| Scalability | Replication and high-availability strategy | TODO |
| Security | TLS or documented reverse-proxy deployment | TODO |

### Priority Rationale

- **High:** Required for data safety, security, operational visibility, and production validation.
- **Medium:** Improves maintainability, API maturity, and operational performance.
- **Low:** Depends on confirmed production scale and deployment requirements.

## Current Validation

The current validation baseline passes:

- `cargo fmt --check`
- `cargo check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features`

Roadmap items must only move to **Implemented Features** after implementation, tests, documentation, and validation are complete.

- Request-size, vector-dimension, collection-name, and `top_k` limits.
- Consistent JSON errors for malformed requests, validation failures, oversized bodies, missing collections, and storage failures.
- HTTP tests covering all request-limit and error cases.

## CI checks for formatting, compilation, tests, Clippy, security advisories, and dependency policy.
See [`docs/testing.md`](docs/testing.md) for the complete testing and security
validation procedure.


