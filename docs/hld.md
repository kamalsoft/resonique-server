# High-Level Design

## Components

| Component | Responsibility |
|---|---|
| `main.rs` | Application entry point |
| `server::run` | Startup orchestration |
| `CollectionManager` | Collection registry and partition routing |
| `StorageEngine` | Storage-root management and path resolution |
| `Wal` | Append-only durability log |
| `Segment` | Vector and metadata persistence |
| `search` | Similarity calculation and filtering |
| HTTP module | JSON API |
| MCP module | Stdio JSON-RPC transport |
| Tests | Functional, security, performance, and health validation |

## Insert flow

```text
Request
  → Deserialize JSON
  → Lock CollectionManager
  → Route vector ID
  → Append WAL record
  → Append segment record
  → Increment insert metric
  → Return response
```

## Search flow

```text
Request
  → Deserialize JSON
  → Select metric
  → Find collection
  → Search every partition
  → Read metadata
  → Apply tag/key-value filters
  → Read vector
  → Calculate score
  → Sort and truncate
  → Increment search metric
  → Return results
```

## Storage layout

```text
storage_root/
├── default_p0.segment
└── default_p0.wal
```

## Reliability considerations

The WAL provides recovery input after startup. Segment indexes are currently held in memory. A production deployment should add framed records, durable index checkpoints, corruption detection, WAL truncation handling, and idempotent replay.
