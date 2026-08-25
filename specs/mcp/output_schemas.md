# MCP Output Schemas

Successful calls use the MCP result envelope:

```text
result.isError: boolean
result.structuredContent: object, when structured output is available
result.content: array<ContentBlock>, when textual output is available
```

Failures use `isError: true` and provide diagnostic content.
