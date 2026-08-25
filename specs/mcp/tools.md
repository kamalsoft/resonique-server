# MCP Tools

MCP is JSON-RPC over standard input/output.

Clients must discover authoritative tool names and schemas through
`tools/list`; names not returned by discovery must not be assumed.

Supported protocol methods include:

- `initialize`
- `ping`
- `tools/list`
- `tools/call`

The exposed tool capabilities correspond to health, collection/partition
inspection, vector insertion, and vector search.
