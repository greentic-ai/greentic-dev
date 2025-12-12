# Worker envelope models

`WorkerRequest`, `WorkerResponse`, and `WorkerMessage` define the shared, domain-agnostic worker envelope used across Greentic services. They mirror the `greentic:worker@1.0.0` WIT package and are intentionally generic: no repo/store/channel concepts leak into these shapes.

## WorkerRequest
- `version: String` – envelope version (set to `"1.0"` for now).
- `tenant: TenantCtx` – tenant context propagated to the worker.
- `worker_id: String` – identifier of the worker being called.
- `correlation_id: Option<String>` – optional correlation handle for tracing.
- `session_id: Option<String>` – optional conversation/session identifier.
- `thread_id: Option<String>` – optional thread identifier for threaded conversations.
- `payload_json: String` – JSON-encoded payload (opaque to the ABI).
- `timestamp_utc: String` – ISO8601 timestamp when the request was created.

## WorkerMessage
- `kind: String` – message kind (for example `text`, `card`, `event`).
- `payload_json: String` – JSON-encoded message payload (opaque to the ABI).

## WorkerResponse
- `version: String` – mirrors the request version (`"1.0"` today).
- `tenant: TenantCtx` – tenant context echoed from the request.
- `worker_id: String` – worker identifier.
- `correlation_id: Option<String>` – optional correlation handle.
- `session_id: Option<String>` – optional session identifier.
- `thread_id: Option<String>` – optional thread identifier.
- `messages: Vec<WorkerMessage>` – zero or more messages produced by the worker.
- `timestamp_utc: String` – ISO8601 timestamp when the response was emitted.

## Usage notes
- Versioning: default to `version = "1.0"` until the ABI evolves.
- Multi-message semantics: responses carry a list, not a stream; upstream callers map each `WorkerMessage` to their own channel/event format.
- Integrations:
  - **Runner** executes workers and returns `WorkerResponse` envelopes.
  - **Messaging** translates channel events into `WorkerRequest` and routes them over NATS/HTTP to workers, then maps `WorkerResponse.messages` back to channel outputs.

These models remain transport- and domain-neutral so new workers (repo assistants, store assistants, brand assistants, etc.) can reuse the same envelope.
