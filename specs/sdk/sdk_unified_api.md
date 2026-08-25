# Unified Consumer API

## Language-neutral interface

```text
health(options?: RequestOptions) -> Result<HealthStatus, ClientError>

list_collections(options?: RequestOptions)
  -> Result<List<CollectionSummary>, ClientError>

list_partitions(
  collection: string,
  options?: RequestOptions
) -> Result<List<PartitionSummary>, ClientError>

insert_vector(
  request: InsertVectorRequest,
  options?: RequestOptions
) -> Result<InsertVectorResponse, ClientError>

search_vectors(
  request: SearchRequest,
  options?: RequestOptions
) -> Result<SearchResponse, ClientError>

metrics(options?: RequestOptions) -> Result<PrometheusText, ClientError>
```

## RequestOptions

```text
timeout: duration, optional
request_id: string, optional
retry_policy: RetryPolicy, optional
```

## Recommended client construction

```text
ClientBuilder
  .endpoint("http://127.0.0.1:3000")
  .timeout(10 seconds)
  .retry_policy(exponential_backoff_with_jitter)
  .build()
```

## Transport mapping

| SDK operation | HTTP | MCP | gRPC | Raw TCP |
|---|---:|---:|---:|---:|
| `health` | Yes | Yes | No | No |
| `list_collections` | Yes | Yes | No | No |
| `list_partitions` | Yes | Yes | No | No |
| `insert_vector` | Yes | Yes | No | No |
| `search_vectors` | Yes | Yes | No | No |
| `metrics` | Yes | No | No | No |
