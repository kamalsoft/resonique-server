# HTTP API Contract

## Connection

| Property | Value |
|---|---|
| Default endpoint | `http://127.0.0.1:3000` |
| Content type | `application/json` |
| Metrics content type | Prometheus text |
| Request size limit | 1 MiB |
| Collection-name limit | 128 characters |
| Vector dimension limit | 4,096 |
| `top_k` range | 1–1,000 |

## Operations

### Health

```text
GET /health
health() -> HealthResponse
```

Response:

```json
{
  "status": "ok"
}
```

### Metrics

```text
GET /metrics
metrics() -> PrometheusText
```

Returns process-local Prometheus metrics.

### List collections

```text
GET /collections
list_collections() -> CollectionListResponse
```

### List partitions

```text
GET /collections/{collection}/partitions
list_partitions(collection: string) -> PartitionListResponse
```

### Insert vector

```text
POST /insert
insert_vector(request: InsertVectorRequest) -> InsertVectorResponse
```

### Search vectors

```text
POST /search
search_vectors(request: SearchRequest) -> SearchResponse
```

## Consumer requirements

- Send `Content-Type: application/json` for JSON requests.
- Reuse connections.
- Configure connect, request, and read timeouts.
- Treat `error.code` as the stable machine-readable value.
- Retry only transport failures and explicitly transient `5xx` responses.
- Do not blindly retry inserts unless idempotency is guaranteed.
