from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

docs = {
    "specs/api/endpoints.md": """# HTTP API Endpoints

Base URL: `http://127.0.0.1:3000`

| Operation | Method | Path | Status |
|---|---|---|---|
| Health check | GET | `/health` | Implemented |
| Metrics | GET | `/metrics` | Implemented |
| List collections | GET | `/collections` | Implemented |
| List partitions | GET | `/collections/{collection}/partitions` | Implemented |
| Insert vector | POST | `/insert` | Implemented |
| Search vectors | POST | `/search` | Implemented |

All endpoints use request/response HTTP semantics and JSON except `/metrics`,
which returns Prometheus text.
""",

    "specs/api/request_schemas.md": """# HTTP Request Schemas

## InsertVectorRequest

```text
collection: string, required
vector_id: uint64, required
vector: array<float>, required
metadata: VectorMetadata, optional
```

## SearchRequest

```text
collection: string, required
query: array<float>, required
top_k: uint32, required; range 1..=1000
metric: cosine | l2, optional
filter: SearchFilter, optional
```

## VectorMetadata

```text
tags: array<string>
timestamp: integer
kv: map<string, string>
```

## SearchFilter

```text
tag: string, optional
key: string, optional
value: string, optional
```

Limits: request body 1 MiB, collection name 128 characters, vector dimension
4,096, and finite numeric vector values.
""",

    "specs/api/response_schemas.md": """# HTTP Response Schemas

## InsertVectorResponse

```text
status: string
vector_id: uint64
```

## SearchResponse

```text
results: array<SearchResult>
```

## SearchResult

```text
vector_id: uint64
score: float
metadata: VectorMetadata
```

## HealthResponse

```text
status: string
```

Topology responses contain collections or partitions as documented by the
corresponding endpoint.
""",

    "specs/api/error_model.md": """# HTTP Error Model

```json
{
  "error": {
    "code": "invalid_vector",
    "message": "Human-readable description"
  }
}
```

| Condition | Status |
|---|---:|
| Malformed JSON | 400 |
| Validation failure | 400 |
| Oversized request | 413 |
| Missing collection or partition | 404 |
| Storage or indexing failure | 500 |

Clients should branch on `error.code`, not `message`.
""",

    "specs/api/best_practices.md": """# HTTP Best Practices

- Reuse connections and configure timeouts.
- Validate dimensions and `top_k` client-side.
- Retry only transient failures.
- Use exponential backoff with jitter.
- Treat inserts as retry-sensitive unless idempotency is guaranteed.
- Do not log vector contents or credentials.
""",

    "specs/mcp/tools.md": """# MCP Tools

MCP is JSON-RPC over standard input/output.

Clients must discover authoritative tool names and schemas through
`tools/list`; names not returned by discovery must not be assumed.

Supported protocol methods include:

- `initialize`
- `ping`
- `tools/list`
- `tools/call`

The exposed tool capabilities correspond to health, collection/partition
inspection, vector insertion, and vector search.
""",

    "specs/mcp/input_schemas.md": """# MCP Input Schemas

Tool arguments use JSON objects.

Vector insertion accepts `collection`, `vector_id`, `vector`, and optional
`metadata`.

Vector search accepts `collection`, `query`, `top_k`, optional `metric`, and
optional filter fields.

Health and topology tools accept their documented required identifiers, if any.
Use `tools/list` as the source of truth.
""",

    "specs/mcp/output_schemas.md": """# MCP Output Schemas

Successful calls use the MCP result envelope:

```text
result.isError: boolean
result.structuredContent: object, when structured output is available
result.content: array<ContentBlock>, when textual output is available
```

Failures use `isError: true` and provide diagnostic content.
""",

    "specs/mcp/error_model.md": """# MCP Error Model

MCP errors may be represented as JSON-RPC errors or tool results with
`isError: true`.

Clients must handle:

- invalid JSON;
- invalid JSON-RPC version;
- missing methods;
- unknown methods;
- invalid tool arguments;
- server and storage failures.
""",

    "specs/mcp/best_practices.md": """# MCP Best Practices

- Complete `initialize` before invoking tools.
- Discover tools and schemas dynamically.
- Preserve request IDs and correlate responses.
- Keep protocol output separate from diagnostics.
- Treat tool errors as data-plane failures.
- Apply bounded timeouts and cancellation.
""",

    "specs/grpc/services.md": """# gRPC Services

gRPC is not implemented in the current server release.

No supported protobuf package, service, listener, or RPC contract is
currently published.
""",

    "specs/grpc/rpc_methods.md": """# gRPC RPC Methods

No RPC methods are currently available.

Future contracts should define health, collection, vector, and search services
only after protobuf compatibility and versioning rules are approved.
""",

    "specs/grpc/request_messages.md": """# gRPC Request Messages

No gRPC request messages are currently supported.

Future messages should use typed fields equivalent to the public HTTP schemas.
""",

    "specs/grpc/response_messages.md": """# gRPC Response Messages

No gRPC response messages are currently supported.

Future responses should define pagination, typed metadata, and compatibility
rules.
""",

    "specs/grpc/error_model.md": """# gRPC Error Model

Not applicable until gRPC is implemented.

The future contract should use standard gRPC status codes and structured
details without exposing internal storage information.
""",

    "specs/grpc/best_practices.md": """# gRPC Best Practices

Future clients should use deadlines, connection reuse, bounded messages,
standard status codes, and backward-compatible protobuf evolution.
""",

    "specs/tcp/framing_protocol.md": """# TCP Framing Protocol

No custom TCP protocol is implemented.

The supported TCP-based transport is HTTP over TCP at the configured HTTP
address.
""",

    "specs/tcp/commands.md": """# TCP Commands

No raw TCP commands are currently defined.

Use the documented HTTP API or MCP stdio transport.
""",

    "specs/tcp/request_frames.md": """# TCP Request Frames

No custom request frame format is supported.
""",

    "specs/tcp/response_frames.md": """# TCP Response Frames

No custom response frame format is supported.
""",

    "specs/tcp/heartbeat.md": """# TCP Heartbeat

No custom TCP heartbeat is defined. HTTP clients should use `/health`.
""",

    "specs/tcp/reconnect_strategy.md": """# TCP Reconnect Strategy

For HTTP-over-TCP clients, reconnect after transport failure using bounded
exponential backoff with jitter. Do not retry non-idempotent writes blindly.
""",

    "specs/tcp/best_practices.md": """# TCP Best Practices

Use HTTP clients rather than implementing a proprietary raw TCP client. Apply
timeouts, TLS through a trusted reverse proxy, connection pooling, and
observability.
""",

    "specs/sdk/sdk_overview.md": """# Client SDK Overview

The SDK should expose one transport-neutral API with HTTP as the primary
transport and MCP as an optional integration transport.

gRPC and raw TCP adapters remain unavailable until their contracts are
implemented.
""",

    "specs/sdk/sdk_signatures.md": """# SDK Signatures

```text
health() -> HealthStatus
list_collections() -> List<Collection>
list_partitions(collection: string) -> List<Partition>
insert_vector(request: InsertVectorRequest) -> InsertVectorResponse
search_vectors(request: SearchRequest) -> SearchResponse
metrics() -> PrometheusText
```
""",

    "specs/sdk/sdk_error_model.md": """# SDK Error Model

```text
ClientError =
  Transport
  | Timeout
  | Serialization
  | Protocol
  | InvalidRequest
  | Server { status, code, message }
```

Errors must preserve stable server error codes where available.
""",

    "specs/sdk/sdk_transport_matrix.md": """# SDK Transport Matrix

| Operation | HTTP | MCP | gRPC | Raw TCP |
|---|---:|---:|---:|---:|
| health | Yes | Yes | No | No |
| list_collections | Yes | Discoverable | No | No |
| list_partitions | Yes | Discoverable | No | No |
| insert_vector | Yes | Discoverable | No | No |
| search_vectors | Yes | Discoverable | No | No |
| metrics | Yes | No | No | No |
""",

    "specs/sdk/sdk_unified_api.md": """# Unified SDK API

The SDK must normalize successful results and errors across available
transports. HTTP is the compatibility baseline. MCP adapters must use dynamic
tool discovery and preserve MCP error semantics.
""",

    "specs/sdk/sdk_best_practices.md": """# SDK Best Practices

- Configure endpoint, timeout, retry policy, and transport explicitly.
- Validate requests before transmission.
- Reuse connections.
- Preserve error codes and request IDs.
- Make retry behavior configurable.
- Avoid caching mutable search results by default.
- Ignore unknown response fields for compatibility.
""",

    "specs/sdk/sdk_backlog.md": """# SDK Backlog

- Publish OpenAPI and MCP schema artifacts.
- Add official SDKs for supported languages.
- Define authentication and TLS configuration.
- Add idempotency support for writes.
- Add pagination and cancellation.
- Add compatibility and contract tests.
""",

    "docs/developer/api-reference.md": "# API Reference\n\nSee [`specs/api`](../../specs/api/).\n",
    "docs/developer/mcp-reference.md": "# MCP Reference\n\nSee [`specs/mcp`](../../specs/mcp/).\n",
    "docs/developer/grpc-reference.md": "# gRPC Reference\n\ngRPC is not currently implemented.\n",
    "docs/developer/tcp-reference.md": "# TCP Reference\n\nOnly HTTP over TCP is currently supported.\n",
    "docs/developer/sdk-guide.md": "# SDK Guide\n\nSee [`specs/sdk`](../../specs/sdk/).\n",

    "docs/product/overview.md": "# Product Overview\n\nResonique Server provides persistent vector storage, metadata, similarity search, HTTP APIs, and MCP integration.\n",
    "docs/product/architecture.md": "# Product Architecture\n\nSee [`docs/architecture.md`](../architecture.md).\n",
    "docs/product/roadmap.md": "# Product Roadmap\n\nFuture work includes gRPC, raw TCP, authentication, TLS, replication, compaction, and migration tooling.\n",
    "docs/product/use-cases.md": "# Use Cases\n\n- Semantic search\n- Metadata-filtered retrieval\n- Agent-integrated vector operations\n- Local and service-based development\n",
    "docs/product/best-practices.md": "# Product Best Practices\n\nUse stable schemas, bounded requests, explicit errors, secure deployment, and tested recovery procedures.\n",
    "docs/product/pitch-deck.md": "# Product Pitch\n\nResonique Server is a persistent, developer-friendly vector search service with HTTP and MCP access.\n",

    "backlog/api_backlog.md": "# API Backlog\n\n- Publish OpenAPI.\n- Add authentication and authorization.\n- Define pagination and idempotency.\n",
    "backlog/mcp_backlog.md": "# MCP Backlog\n\n- Publish authoritative tool schemas.\n- Add protocol compatibility tests.\n- Document tool versioning.\n",
    "backlog/grpc_backlog.md": "# gRPC Backlog\n\n- Define protobuf contracts.\n- Implement typed services.\n- Add unary and streaming compatibility tests.\n",
    "backlog/tcp_backlog.md": "# TCP Backlog\n\n- Do not implement raw TCP without a performance requirement.\n- Define framing, heartbeat, and reconnect contracts if required.\n",
    "backlog/sdk_backlog.md": "# SDK Backlog\n\n- Implement official language SDKs.\n- Add transport adapters.\n- Add contract, retry, and compatibility tests.\n",

    "meta/prompts/extract-all-definitions.md": "# Prompt: Extract Public Definitions\n\nExtract only public endpoint, schema, transport, and compatibility definitions. Never include proprietary implementation details.\n",
    "meta/prompts/generate-sdk.md": "# Prompt: Generate SDK\n\nGenerate transport-neutral SDK signatures from approved public specifications only.\n",
    "meta/prompts/generate-backlog.md": "# Prompt: Generate Backlog\n\nConvert missing public contracts and operational requirements into prioritized backlog items.\n",
    "meta/prompts/generate-docs.md": "# Prompt: Generate Documentation\n\nGenerate developer and product documentation from approved public specifications.\n",
    "meta/prompts/generate-client-spec.md": "# Prompt: Generate Client Specification\n\nProduce schemas, signatures, error models, transport mappings, and best practices without exposing implementation details.\n",
}

for name, content in docs.items():
    path = ROOT / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content.rstrip() + "\n", encoding="utf-8")

print(f"Generated {len(docs)} documentation files.")