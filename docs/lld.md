# Low-Level Design

## Configuration models

- `ServerConfig`
  - `storage_root: String`
  - `collections: Vec<CollectionManifest>`
- `CollectionManifest`
  - `name`
  - `partitions`
- `PartitionConfig`
  - `name`
  - `hash_range`

Runtime models remove serialization concerns and are used by the server state.

## Collection manager

`CollectionManager` stores:

```rust
HashMap<String, CollectionState>
```

`route_partition()` selects a partition when:

```text
hash_range.start <= vector_id <= hash_range.end
```

The current routing implementation uses the vector ID directly as its hash value.

## Segment format

A segment begins with a JSON header containing:

- magic value
- version
- entry count
- index offset
- payload offset

Each inserted record writes:

1. Raw vector bytes.
2. Serialized metadata JSON.
3. An in-memory `SegmentIndexEntry`.

The current format does not persist vector dimensions or metadata lengths. Future framing should add both lengths to make recovery and corruption detection deterministic.

## WAL format

Each line is a JSON object containing:

- `vector_id`
- byte-array payload
- `VectorMetadata`

`Wal::replay()` reads non-empty lines and deserializes them into `WalRecord` values.

## Search

Supported metrics:

- `Cosine`: higher score is better.
- `L2`: lower distance is better.

Search reads candidate metadata before reading the vector. Filters support:

- a required tag
- a metadata key/value pair

## Error handling

The current public implementation uses `anyhow::Result` for storage and startup operations. HTTP handlers currently return JSON status values. A future typed API error layer should map validation, routing, storage, and internal failures to appropriate HTTP status codes.

## Concurrency

The HTTP and MCP transports share:

```rust
Arc<tokio::sync::Mutex<CollectionManager>>
```

The mutex protects mutable segment files, WAL files, and collection state. A read/write lock or partition-level locks may improve concurrency for production workloads.
