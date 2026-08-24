# Testing and Security Validation

## Unit and integration tests

```bash
cargo test --all-targets --all-features
```

Run tests with logs:

```bash
RUST_LOG=debug cargo test --all-targets --all-features -- --nocapture
```

## Performance tests

Run the existing performance tests in release mode:

```bash
cargo test --release performance -- --nocapture
```

Performance tests must report insert throughput and search latency. Results are
environment-dependent and should be compared using the same machine,
configuration, dataset, and build profile.

## Static and dependency checks

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo audit
cargo deny check
```

## API security tests

Start the server in one terminal:

```bash
cargo run
```

Run the security checks in another terminal:

```bash
./scripts/security-test.sh
```

These checks cover malformed JSON, oversized requests, invalid vectors,
invalid `top_k`, unknown collections, and path-like collection input.

## Penetration testing

Automated tests are not a substitute for a complete penetration test. Before
network deployment, test in an isolated environment using:

- OWASP ZAP or Burp Suite for HTTP traffic;
- Nmap for exposed-port verification;
- TLS and certificate validation tools;
- authenticated and unauthenticated request scenarios;
- rate-limit and resource-exhaustion tests;
- SSRF and path-traversal tests when remote ingestion is enabled.

Do not run destructive tests against production data.

## Required quality gate

A change is ready for review only when unit tests, integration tests,
performance tests, security tests, formatting, Clippy, dependency auditing,
license checks, and coverage review are complete.

## Coverage

Generate a text summary:

```bash
cargo llvm-cov --lib --all-features \
  --show-missing-lines --summary-only
```

Generate and open the HTML report on macOS:

```bash
cargo llvm-cov --lib --all-features --html --open
```

The report is written to:

```text
target/llvm-cov/html/index.html
```

The current baseline contains **84 passing tests**. Coverage includes HTTP,
MCP, storage, WAL, search, configuration, security, integration, and
performance tests.

`server::run()` transport orchestration is intentionally not exercised by
starting real MCP stdin or HTTP listeners. Test manager construction and
transport handlers independently instead.