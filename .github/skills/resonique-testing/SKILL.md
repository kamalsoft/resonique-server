---
name: resonique-testing
description: Test Resonique correctness, performance, and parallel request handling.
---

# Resonique Testing

- Add unit tests for isolated logic.
- Add integration tests for API, storage, and search behavior.
- Add regression tests for every bug fix.
- Use temporary directories for test data.
- Avoid committed `data/` files and fixed network ports.
- Test invalid input and failure paths.

## Performance and parallelism

- Add performance tests for insert, indexing, and search operations.
- Measure elapsed time using `std::time::Instant`.
- Keep performance assertions stable; avoid strict machine-dependent thresholds.
- Test concurrent requests using scoped threads or an async task pool already used by the project.
- Verify concurrent operations do not panic, lose records, corrupt the WAL, or produce inconsistent search results.
- Use unique test collections or isolated temporary storage for each test.
- Run performance tests separately when they are expensive.

## Validation

```bash
cargo fmt --check
cargo check
cargo test
cargo test --release
```

For focused performance tests:

```bash
cargo test --release performance -- --nocapture
```