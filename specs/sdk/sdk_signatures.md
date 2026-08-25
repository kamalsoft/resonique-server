# SDK Signatures

```text
health() -> HealthStatus
list_collections() -> List<Collection>
list_partitions(collection: string) -> List<Partition>
insert_vector(request: InsertVectorRequest) -> InsertVectorResponse
search_vectors(request: SearchRequest) -> SearchResponse
metrics() -> PrometheusText
```
