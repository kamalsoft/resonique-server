# TCP/IP Connectivity

## Current Transport

Resonique Server exposes its HTTP API over TCP.

Default address:

```text
127.0.0.1:3000
```

Configure the listener with:

```bash
RESONIQUE_HTTP_ADDR=0.0.0.0:3000 cargo run
```

Available endpoints:

- `GET /health`
- `GET /metrics`
- `GET /collections`
- `GET /collections/:collection/partitions`
- `POST /insert`
- `POST /search`

## Connectivity Options

| Option | Status | Intended use |
|---|---|---|
| HTTP/REST over TCP | Implemented | General clients and SDKs |
| HTTPS over TCP | Planned | Secure network access |
| gRPC over TCP | Planned | Typed, high-throughput service communication |
| WebSocket over TCP | Planned | Bidirectional streaming |
| Custom binary TCP protocol | Deferred | Only if profiling justifies it |

## Security

Binding to `0.0.0.0` exposes the service on all network interfaces. Use a
firewall, private network, reverse proxy, authentication, and TLS before
exposing the service outside a trusted environment.

## Recommended Progression

1. Configure the existing HTTP listener.
2. Add authentication and TLS.
3. Publish client SDKs.
4. Add gRPC for service-to-service workloads.
5. Add WebSockets for streaming use cases.
6. Consider a custom binary protocol only after performance testing.