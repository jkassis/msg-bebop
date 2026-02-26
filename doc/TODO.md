# Courier TODO

## Implementation TODOs

### P0

- [x] Implement `IdempotencyStrategy` interface in Rust and wire it into `rx()`.
- [x] Keep receipt dedup as the first built-in strategy under `IdempotencyStrategy::Receipts`.
- [x] Add an `ObservabilityRecorder` interface and emit stable events from `tx`, `tick`, and `rx` paths.
- [x] Add a no-op recorder implementation so integrations are optional-by-default.
- [ ] Add `Msg.version` to all language models and ensure deserialization defaults to `1` when missing. (Rust done; Go/TS pending)
- [ ] Add ACK validation fields (`msg_id`, `from_id`, `to_id`, `version`) and reject malformed/mismatched ACKs. (Rust done; Go/TS pending)
- [x] Define/implement multi-recipient ACK progress semantics by removing the validated ACK `to_id` from message recipient targets so retries only target unacked recipients.

### P1

- [x] Add expiration hook support and provide a DLQ-oriented reference implementation.
- [ ] Add retry/idempotency guardrails, including ACK-path correctness checks and receipt strategy validation (`keep_receipt >= max_retry_horizon` when statically knowable).
- [ ] Define typed error categories (`retryable`, `terminal`, `caller_bug`) consistently across Rust/Go/TS.
- [ ] Add docs and examples for embedder-managed ordering (custom ordering fields + receiver-side ordered queueing).

### P2

- [ ] Build cross-language conformance suite with shared scenarios and expected outcomes.
- [ ] Add deterministic replay test cases (fixed tick stream, fixed policy functions, deterministic IDs).
- [ ] Add operator runbooks for backlog spikes, poison messages, DB pressure, and ack-loss scenarios.
