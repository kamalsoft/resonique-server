# TCP Reconnect Strategy

For HTTP-over-TCP clients, reconnect after transport failure using bounded
exponential backoff with jitter. Do not retry non-idempotent writes blindly.
