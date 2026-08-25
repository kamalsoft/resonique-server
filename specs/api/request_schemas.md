# HTTP Request Schemas

## InsertVectorRequest

| Field | Type | Required | Constraints |
|---|---|---:|---|
| `collection` | string | Yes | Non-empty, maximum 128 characters |
| `vector_id` | uint64 | Yes | Must resolve to a configured partition |
| `vector` | array<float> | Yes | Non-empty, finite, maximum 4,096 values |
| `metadata` | VectorMetadata | No | Defaults to empty metadata |

Example:

```json
{
  "collection": "default",
  "vector_id": 42,
  "vector": [0.12, -0.34, 0.56],
  "metadata": {
    "tags": ["support", "technical"],
    "timestamp": 1720000000,
    "kv": {
      "tenant": "acme",
      "source": "manual"
    }
  }
}
```

## SearchRequest

| Field | Type | Required | Constraints |
|---|---|---:|---|
| `collection` | string | Yes | Existing collection |
| `query` | array<float> | Yes | Non-empty, finite |
| `top_k` | uint32 | Yes | 1 through 1,000 |
| `metric` | `cosine` \| `l2` | No | Server default if omitted |
| `filter` | SearchFilter | No | Optional metadata filter |

## SearchFilter

```text
tag: string, optional
key: string, optional
value: string, optional
```

A filter containing `key` without `value` checks key presence. A filter
containing both `key` and `value` checks key/value equality.
