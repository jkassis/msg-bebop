# Courier TODO

## Implementation TODOs

### P0

- [ ] Implement `IdempotencyStrategy` interface in Rust and wire it into `rx()`.
- [ ] Keep receipt dedup as the first built-in strategy under `IdempotencyStrategy::Receipts`.
- [ ] Add an `ObservabilityRecorder` interface and emit stable events from `tx`, `tick`, and `rx` paths.
- [ ] Add a no-op recorder implementation so integrations are optional-by-default.
- [ ] Add `Msg.version` to all language models and ensure deserialization defaults to `1` when missing.
- [ ] Add ACK validation fields (`msg_id`, `from_id`, `to_id`, `version`) and reject malformed/mismatched ACKs.
- [ ] Define/implement multi-recipient ACK progress semantics by removing the validated ACK `to_id` from message recipient targets so retries only target unacked recipients.

### P1

- [ ] Add expiration hook support and provide a DLQ-oriented reference implementation.
- [ ] Add retry/idempotency guardrails, including ACK-path correctness checks and receipt strategy validation (`keep_receipt >= max_retry_horizon` when statically knowable).
- [ ] Define typed error categories (`retryable`, `terminal`, `caller_bug`) consistently across Rust/Go/TS.
- [ ] Add docs and examples for embedder-managed ordering (custom ordering fields + receiver-side ordered queueing).

### P2

- [ ] Build cross-language conformance suite with shared scenarios and expected outcomes.
- [ ] Add deterministic replay test cases (fixed tick stream, fixed policy functions, deterministic IDs).
- [ ] Add operator runbooks for backlog spikes, poison messages, DB pressure, and ack-loss scenarios.

## Conformance Suite Scope

- [ ] Golden scenarios: happy path, ack loss, sender crash before/after commit, receiver crash before/after ack, duplicate replay after dedup window.
- [ ] Invariant assertions: at-least-once delivery, no pre-commit visibility, one-attempt-per-message-per-tick, duplicate suppression by configured strategy.
- [ ] Language parity checks for wire format and state transitions (`Msg`, `Pact`, `Receipt`, events).

## Open Questions (Decision Needed)

## Decisions Captured

- Observability payload baseline is option B, with retry/latency fields and monotonic/injected timing (avoid hot-path wall-clock calls).
- ACK validation requires correlation fields and malformed ACK rejection.
- Message version evolution uses configurable compatibility windows (strategy B).
- Runbooks will be split by incident class under `doc/runbooks/` (strategy B).
- Multi-recipient ACK progress uses recipient-list shrinking: on validated ACK, remove the acking `to_id` from the message's target recipient IDs so future retries skip that recipient.
- ACK timeout/retry for partial success uses one shared pact per message (Option A) to avoid per-recipient write amplification.
