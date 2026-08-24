# Execution Guide

## Prerequisites

- Rust toolchain compatible with the project
- Cargo
- macOS, Linux, or Windows

Verify the environment:

```bash
rustc --version
cargo --version
```

## Build and Run

Build the application:

```bash
cargo build
```

Start the server with the default configuration:

```bash
cargo run
```

The default HTTP endpoint is:

```text
http://127.0.0.1:3000
```

Storage is loaded from the configured application data directory.

## HTTP over TCP

Check server health:

```bash
curl -i http://127.0.0.1:3000/health
```

Insert a vector:

```bash
curl -i -X POST http://127.0.0.1:3000/insert \
  -H 'content-type: application/json' \
  -d '{"collection":"default","vector_id":1,"vector":[0.1,0.2],"metadata":{"tags":["test"],"timestamp":0,"kv":{}}}'
```

Search vectors:

```bash
curl -i -X POST http://127.0.0.1:3000/search \
  -H 'content-type: application/json' \
  -d '{"collection":"default","query":[0.1,0.2],"top_k":5}'
```

List collections:

```bash
curl -i http://127.0.0.1:3000/collections
```

Test the TCP listener:

```bash
nc -vz 127.0.0.1 3000
```

### Configure the HTTP address

Set `RESONIQUE_HTTP_ADDR` before starting the application:

```bash
RESONIQUE_HTTP_ADDR=0.0.0.0:3000 cargo run
```

Binding to `0.0.0.0` exposes the service on all network interfaces. Use TLS,
authentication, firewall rules, and a private network before enabling remote
access.

## MCP over stdio

MCP currently uses standard input and standard output rather than TCP.

List available tools:

```bash
printf '%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' \
  | cargo run
```

Call the health tool:

```bash
printf '%s\n' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"health_mcp","arguments":{}}}' \
  | cargo run
```

MCP diagnostic output must remain separate from JSON-RPC responses.

## gRPC

gRPC is not currently implemented. There is no gRPC listener or valid gRPC
command for this version.

When implemented, the service should expose a documented Protocol Buffers
contract and listen on a separate TCP port, such as:

```text
127.0.0.1:50051
```

## Automated Validation

Format the source:

```bash
cargo fmt
```

Run all tests:

```bash
cargo test --all-targets --all-features
```

Run Clippy:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

Run dependency and license checks:

```bash
cargo deny check
```

Run the complete validation sequence as separate commands:

```bash
cargo fmt --check
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo deny check
```

## Shutdown

Stop the development server with:

```text
Ctrl-C
```

The application should be stopped gracefully before its storage directory is
moved, backed up, or removed.

## Execution Modes

| Mode | Transport | Current status | Default address |
|---|---|---|---|
| HTTP API | HTTP over TCP | Available | `127.0.0.1:3000` |
| MCP | stdio | Available | Not applicable |
| Raw TCP protocol | Custom TCP | Not implemented | Not applicable |
| gRPC | HTTP/2 over TCP | Not implemented | Not applicable |

## Current Status

| Mode | Status | Test method |
|---|---|---|
| HTTP over TCP | Available | `curl`, `nc` |
| MCP over stdio | Available | JSON-RPC through stdin |
| Raw custom TCP | Not implemented | — |
| gRPC over TCP | Not implemented | — |

## Testing and Security

See [`docs/testing.md`](docs/testing.md) for unit, integration, performance,
security, dependency, and penetration-testing procedures.

## Verification

Behavior described in this document must be validated using the test and
security procedures in [`testing.md`](testing.md).