# Compatibility Policy

Clients must:

- ignore unknown response fields;
- preserve unknown enum values where possible;
- use stable error codes;
- avoid depending on response ordering except search ranking;
- use explicit client and server version headers when introduced;
- fail closed when required fields are missing.
