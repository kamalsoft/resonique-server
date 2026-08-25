# Resonique Public Specifications

This directory contains the versioned, consumer-facing contracts for Resonique
Server. It describes supported behavior only; unsupported transports are
explicitly marked as unavailable.

## Specifications

- [HTTP API](api/endpoints.md)
- [MCP Tools](mcp/tools.md)
- [gRPC](grpc/services.md)
- [TCP](tcp/framing_protocol.md)
- [Client SDK](sdk/sdk_overview.md)

## Consumer workflow

1. Select a supported transport.
2. Configure the server endpoint.
3. Discover or validate the server contract.
4. Construct a typed request.
5. Apply timeout and retry policies.
6. Decode the typed response.
7. Handle stable error codes.
8. Record request IDs and latency metrics.

## Support status

| Transport | Status |
|---|---|
| HTTP over TCP | Supported |
| MCP over stdio | Supported |
| gRPC | Not implemented |
| Raw TCP | Not implemented |