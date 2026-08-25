# HTTP Error Model

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
