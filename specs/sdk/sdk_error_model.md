# SDK Error Model

```text
ClientError =
  Transport
  | Timeout
  | Serialization
  | Protocol
  | InvalidRequest
  | Server { status, code, message }
```

Errors must preserve stable server error codes where available.
