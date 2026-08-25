# MCP Input Schemas

Tool arguments use JSON objects.

Vector insertion accepts `collection`, `vector_id`, `vector`, and optional
`metadata`.

Vector search accepts `collection`, `query`, `top_k`, optional `metric`, and
optional filter fields.

Health and topology tools accept their documented required identifiers, if any.
Use `tools/list` as the source of truth.
