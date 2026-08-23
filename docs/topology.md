# Deployment Topology

## Current single-node topology

```text
┌──────────────────────────────┐
│        Resonique process      │
│                              │
│  HTTP :3000                  │
│  MCP stdin/stdout            │
│                              │
│  CollectionManager           │
│      ├── Partition p0        │
│      └── Partition p1        │
│                              │
│  Local filesystem             │
│      ├── *.segment            │
│      └── *.wal                │
└──────────────────────────────┘
```

## Recommended production placement

- Run the server under a process supervisor.
- Mount `storage_root` on durable local storage.
- Restrict filesystem permissions to the service account.
- Place an authenticated reverse proxy in front of HTTP.
- Export logs to a centralized logging system.
- Back up WAL and segment files consistently.

## Future multi-node topology

```text
Clients
   │
Load balancer
   │
┌──┴──────────────┐
│                 │
Node A            Node B
│                 │
Shard ranges      Shard ranges
│                 │
Durable storage   Durable storage
```

Future distributed operation requires ownership assignment, replication, membership, rebalancing, and cross-node search aggregation.
