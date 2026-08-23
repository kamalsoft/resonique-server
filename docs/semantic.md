# Resonique Server — Semantic Document

## 1. Purpose

Resonique Server is a persistent vector-search service. It stores vectors and
metadata, exposes insertion and similarity-search APIs, and restores indexed
data from its write-ahead log (WAL) after restart.

## 2. Core Concepts

| Concept | Meaning |
|---|---|
| Collection | Logical namespace containing vector partitions. |
| Partition | Storage unit selected by vector ID hash range. |
| Vector | Numeric embedding associated with a unique vector ID. |
| Metadata | Tags, timestamp, and arbitrary key-value attributes. |
| WAL | Durable append-only record used for recovery. |
| Segment | In-memory/searchable representation of persisted vectors. |
| Metric | Similarity calculation, currently including cosine similarity. |

## 3. Data Model

A vector record contains:

- `vector_id`: unique unsigned integer.
- `vector`: non-empty finite floating-point values.
- `metadata.tags`: list of textual tags.
- `metadata.timestamp`: application-defined timestamp.
- `metadata.kv`: arbitrary string key-value attributes.

Collections contain one or more partitions. Each partition has:

- a name;
- an inclusive hash-range representation;
- a node identifier;
- a WAL;
- a searchable segment.

## 4. Request Constraints

The HTTP layer enforces:

| Field | Constraint |
|---|---|
| Request body | Maximum 1 MiB |
| Collection name | Non-empty, maximum 128 characters |
| Vector | Non-empty, maximum 4,096 dimensions |
| Vector values | Must be finite |
| `top_k` | Between 1 and 1,000 inclusive |

Malformed JSON and invalid content types are rejected before business logic
executes.

## 5. HTTP Semantics

### Health

`GET /health`

Returns service health information.

### Metrics

`GET /metrics`

Returns operational counters, including insertion and search counts.

### Insert

`POST /insert`

Persists a vector to the collection partition selected by its vector ID.

Processing order:

1. Decode and validate the request.
2. Resolve the target collection.
3. Resolve the target partition.
4. Append the record to the WAL.
5. Insert the record into the searchable segment.
6. Increment the insertion counter.

### Search

`POST /search`

Returns the highest-scoring matching vectors.

Processing order:

1. Decode and validate the request.
2. Resolve the collection.
3. Search each collection partition.
4. Merge partition results.
5. Sort results by descending score.
6. Truncate to `top_k`.
7. Increment the search counter.

### Collections

Collection and partition inspection endpoints expose configured topology without
modifying stored data.

## 6. Error Semantics

Errors use a consistent JSON structure:

```json
{
  "error": {
    "code": "invalid_vector",
    "message": "vector must not be empty"
  }
}
```

Expected categories include:

| Condition | HTTP status |
|---|---:|
| Malformed JSON | 400 |
| Validation failure | 400 |
| Oversized request | 413 |
| Missing collection | 404 |
| Missing partition | 404 |
| Storage or indexing failure | 500 |

Clients should branch on the stable `error.code`, not the human-readable
message.

## 7. Persistence and Recovery

Writes are made durable through the WAL before being indexed. On startup,
`CollectionManager` reconstructs collection state and replays WAL records into
segments. Therefore, a successfully acknowledged insert must be recoverable
after process restart.

## 8. Partition Routing

A vector is routed using its vector ID and the configured partition hash range.
Routing must resolve exactly one valid partition. Requests that cannot resolve a
collection or partition return a not-found error.

## 9. Operational Counters

The service maintains atomic counters for:

- successful insert operations;
- successful search operations.

Counters are process-local and are intended for operational visibility rather
than durable accounting.

## 10. Security and Reliability Requirements

- Reject path traversal in storage paths.
- Bound request bodies before deserialization.
- Reject non-finite vector values.
- Avoid exposing internal storage errors directly to clients.
- Preserve WAL-before-index ordering.
- Keep validation behavior consistent across all API handlers.

## 11. Testing Contract

The test suite must cover:

- unit behavior for segments, indexes, and WAL operations;
- end-to-end insert, search, and WAL replay;
- path traversal prevention;
- malformed JSON;
- oversized request bodies;
- invalid vectors;
- invalid `top_k`;
- invalid collection names;
- unknown collections;
- HTTP error response consistency.

Any change to request validation, routing, persistence, search metrics, or error
codes must add or update the corresponding tests.