# MCP Error Model

MCP errors may be represented as JSON-RPC errors or tool results with
`isError: true`.

Clients must handle:

- invalid JSON;
- invalid JSON-RPC version;
- missing methods;
- unknown methods;
- invalid tool arguments;
- server and storage failures.
