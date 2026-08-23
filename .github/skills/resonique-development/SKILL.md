---
name: resonique-development
description: Develop and maintain the Resonique Rust server.
---

# Resonique Development

- Use Rust and Cargo; do not introduce .NET patterns.
- Keep HTTP and API code in `src/server`.
- Keep persistence, WAL, segments, indexing, and search in `src/storage`.
- Keep MCP integration in `src/mcp`.
- Reuse existing abstractions before adding new modules.
- Return meaningful `Result` errors; avoid server-side panics.
- Validate external input at API boundaries.
- Run `cargo fmt` and `cargo check` after changes.
- Update `docs/` when architecture or behavior changes.
- Never commit `target/`, generated logs, or local data files.

## Roadmap completion policy

When implementing a roadmap item:

1. Implement the complete feature, not a partial scaffold.
2. Add unit and integration tests.
3. Add security, failure-path, and concurrency tests where applicable.
4. Update the relevant API and architecture documentation.
5. Run the complete validation checklist.
6. Move the item from `Roadmap` to `Implemented Features` only when all acceptance criteria pass.
7. Do not ask for confirmation before synchronizing the README.
8. Keep incomplete work marked `TODO`.

MCP roadmap work must preserve the existing stdio transport. gRPC work must include protobuf definitions, server implementation, error handling, compatibility rules, and client/integration tests.

## Standing implementation policy

- Implement low-priority roadmap items incrementally.
- Complete one feature before starting the next.
- For every completed feature:
  1. Add implementation, tests, documentation, and configuration.
  2. Run the full validation suite twice in separate executions:
     `cargo fmt --check`, `cargo check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --all-features`.
  3. Move the item from `Roadmap` to `Implemented Features` only after both runs pass.
  4. Leave the item as `TODO` if either validation run fails.
- Do not mark scaffolding or partial implementations as complete.

### Low-priority order

1. TLS/reverse-proxy deployment documentation and configuration.
2. Replication and high-availability design.
3. Horizontal scaling and partition distribution.
4. Upgrade and migration runbooks.