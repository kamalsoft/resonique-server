# HTTP Response Schemas

## InsertVectorResponse

```json
{
  "status": "inserted",
  "vector_id": 42
}
```

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

For `cosine`, larger scores rank first. For `l2`, smaller distances rank
first.

## CollectionListResponse

```text
collections: array<CollectionSummary>
```

## PartitionListResponse

```text
partitions: array<PartitionSummary>
```

SDKs should tolerate additive response fields for forward compatibility.
