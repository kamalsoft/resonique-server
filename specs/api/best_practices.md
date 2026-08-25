# HTTP Best Practices

- Reuse connections and configure timeouts.
- Validate dimensions and `top_k` client-side.
- Retry only transient failures.
- Use exponential backoff with jitter.
- Treat inserts as retry-sensitive unless idempotency is guaranteed.
- Do not log vector contents or credentials.
