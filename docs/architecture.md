# Architecture

## Overview

Resonique Server is organized into five primary layers:

1. **Runtime** — starts Tokio, loads configuration, initializes logging, and starts transports.
2. **API** — exposes HTTP handlers and MCP stdio processing.
3. **Server state** — manages collections and routes vector IDs to partitions.
4. **Storage** — writes WAL records, stores segment payloads, and rebuilds indexes.
5. **Search** — reads vectors and metadata, applies filters, calculates similarity, and returns top-K results.

```text
Client
  │
  ├── HTTP ──> Axum handlers
  │              │
  │              ▼
  │       CollectionManager
  │              │
  │       PartitionState
  │          ┌───┴────┐
  │          ▼        ▼
  │         WAL    Segment
  │                    │
  │                    ▼
  │                 Search
  │
  └── MCP stdio ──> MCP transport
```

## Runtime lifecycle

1. Initialize tracing.
2. Load `config.json`.
3. Create and initialize `StorageEngine`.
4. Convert collection manifests into runtime models.
5. Open segment and WAL files for every partition.
6. Replay WAL files into in-memory indexes.
7. Spawn MCP transport.
8. Start the HTTP listener.

## Consistency model

An insert first appends to the WAL and then writes to the segment. This establishes a durability-first order, but recovery and duplicate-record handling remain areas for further hardening.
