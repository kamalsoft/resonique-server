---
name: resonique-server
description: Guidelines for developing the Resonique Rust search server.
---

# Resonique Server Development

## Project context

- Language: Rust
- Build system: Cargo
- Entry point: `src/main.rs`
- Library root: `src/lib.rs`
- HTTP server: `src/server`
- Storage engine: `src/storage`
- Data models: `src/model`
- MCP integration: `src/mcp`
- Tests: `src/tests`
- Persistent data: `data/`

## Coding rules

- Follow idiomatic Rust and run `cargo fmt`.
- Do not introduce `.NET` or unrelated framework dependencies.
- Prefer existing modules and abstractions before adding new ones.
- Use `Result` for recoverable errors and preserve useful error context.
- Avoid panics in server and storage code.
- Keep HTTP handling in `src/server`.
- Keep persistence and indexing logic in `src/storage`.
- Validate external input at API boundaries.
- Protect concurrent access to collections, indexes, WAL, and segments.
- Do not commit files under `target/`, `data/`, or local logs.

## Validation

Run the following before completing changes:

```bash
cargo fmt --check
cargo check
cargo test
```

For performance-sensitive changes, also run:

```bash
cargo test --release
```

## Change guidance

- Update relevant documentation under `docs/` when architecture changes.
- Add unit tests for isolated logic.
- Add integration tests for HTTP, storage, and search behavior.
- Preserve backward compatibility for existing APIs and configuration.
- Review WAL recovery and segment persistence when modifying storage code.