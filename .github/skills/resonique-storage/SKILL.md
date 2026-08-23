---
name: resonique-storage
description: Safely modify Resonique persistence and search storage.
---

# Resonique Storage

Before changing storage code, inspect:

- `src/storage/wal.rs`
- `src/storage/segment.rs`
- `src/storage/index.rs`
- `src/storage/search.rs`
- `src/server/collection.rs`

Rules:

- Preserve WAL recovery and replay behavior.
- Consider crashes during writes and partial records.
- Preserve segment and index consistency.
- Do not silently discard corrupted data.
- Handle concurrent access safely.
- Add tests for restart, recovery, empty collections, and missing data.
- Measure performance before optimizing search or indexing.