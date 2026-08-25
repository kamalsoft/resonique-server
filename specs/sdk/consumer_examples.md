# Consumer Examples

## Insert

```text
client.insert_vector({
  collection: "default",
  vector_id: 42,
  vector: [0.12, -0.34, 0.56]
})
```

## Search

```text
client.search_vectors({
  collection: "default",
  query: [0.12, -0.34, 0.56],
  top_k: 5,
  metric: "cosine"
})
```
