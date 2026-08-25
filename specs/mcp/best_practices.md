# MCP Best Practices

- Complete `initialize` before invoking tools.
- Discover tools and schemas dynamically.
- Preserve request IDs and correlate responses.
- Keep protocol output separate from diagnostics.
- Treat tool errors as data-plane failures.
- Apply bounded timeouts and cancellation.
