# Courier Architecture Review: Strengths, Weaknesses, Concerns, Gaps, Improvements

## Strengths

- Clear core contract: transactional outbox + asynchronous delivery + retry until ack/expiry.
- Correctly centers on at-least-once semantics and makes idempotency explicit.
- Strong separation of concerns: storage, transport, retry policy are all pluggable.
- Logical tick model is flexible and supports deterministic runtimes as well as wall-clock services.
- Key invariants and non-guarantees are explicitly documented.
- Failure-mode catalog exists and maps crashes to expected recovery behavior.
- Prefix-based key namespace supports multi-tenant use in one backing store.
- Explicit warning on dual-write problem and how Courier addresses it with shared DB transactions.

## Weaknesses

- Critical behavior is under-specified around ACK semantics (authenticity, correlation, replay handling).
- Tick processing is effectively single-threaded by contract; no built-in lease/lock/ownership model for horizontal workers.
- Receipt-based dedup is bounded-window and easy to misconfigure vs retry horizon.
- Backpressure controls are missing (no batch limits, no per-tick budget, no queue-depth guardrails).
- Expiration handling is described but not implemented, leaving dead-letter/operational handling undefined.
- `Context`-threaded transaction ownership is runtime-convention-heavy and prone to misuse.
- Transport abstraction is broad, but delivery/security expectations at the boundary are not standardized.
- Implementation parity across Rust/Go/TS is not yet present, so semantic drift risk is high.

## Concerns

- Duplicate side effects remain likely if receiver idempotency state and handler side effects are not committed atomically.
- Lost-or-spoofed ACK risk: current design does not define sender-side verification that an ACK is legitimate.
- Burst risk from large tick jumps can create latency spikes, retry storms, and downstream overload.
- Receipt cleanup piggy-backed on `tick()` can lag indefinitely if tick frequency drops.
- Hot-key/index pressure likely on `te:{tick}:...` and potentially on sender/receiver message IDs in high-throughput workloads.
- No documented poison-message strategy beyond generic expiry mention.
- Unknown behavior under partial transport success for multi-recipient metadata (`to_ids`) because fanout is delegated.
- If caller commits after `courier.tx()` partial failure, recovery/corruption remediation path is not defined.

## Gaps

- No precise API/error model for `tx`, `tick`, and `rx` across languages (typed error taxonomy, retryable vs terminal).
- No explicit ordering model by scope (per-message, per-sender, per-recipient) beyond “no global ordering.”
- No idempotency-key scope/TTL contract for clients and no migration plan to pluggable idempotency.
- No observability spec: required logs/metrics/traces/correlation IDs and cardinality guidance are missing.
- No security model for transport/plugin boundaries (authn/authz, ACLs, replay protection, integrity).
- No capacity model for scan complexity, compaction cadence, receipt retention cost, or SLO-driven sizing.
- No formal recovery runbooks (DB full, backlog surge, stuck retries, large replay after outage).
- No compatibility/versioning strategy for message schema evolution across languages.
- No conformance test suite to guarantee equivalent semantics across Rust/Go/TS implementations.

## Improvements (Prioritized)

1. Define a normative protocol contract:
   - ACK schema and correlation requirements.
   - ACK authenticity/integrity checks.
   - Typed errors with retryability semantics.

2. Add operational backpressure controls:
   - `tick()` max batch size and max processing duration.
   - Per-destination concurrency limits.
   - Explicit retry budget and circuit-breaker hooks.

3. Implement expiration and DLQ pipeline:
   - Required expiration hook interface.
   - Standard dead-letter record schema.
   - Re-drive tooling and replay guardrails.

4. Harden idempotency strategy:
   - Ship pluggable idempotency interface.
   - Add safe defaults that bind dedup window >= retry horizon.
   - Provide monotonic-sequence strategy for long-lived streams.

5. Introduce distributed tick ownership option:
   - DB-backed lease/lock for active tick worker.
   - Fencing token or epoch to prevent split-brain processing.

6. Publish observability baseline:
   - Metrics: send attempts, ack latency, retries, expirations, backlog depth, dedup hits.
   - Structured events with stable names.
   - Trace propagation and correlation ID requirements.

7. Define storage performance guidance:
   - Key layout recommendations to reduce hot prefixes.
   - Scan/compaction expectations and retention policies.
   - Capacity planning formulas by throughput and retry profile.

8. Standardize cross-language behavior:
   - Spec-first conformance tests shared across Rust/Go/TS.
   - Golden scenario matrix (crash points, retries, dedup-window boundary, ack loss).

9. Strengthen transaction-safety ergonomics:
   - Rust compile-time safe transaction wrapper (where feasible).
   - Runtime guardrails in Go/TS with explicit fatal errors when tx context is missing.

10. Add runbooks and failure drills:
    - Recovery procedures for backlog burst and outage replay.
    - Operator actions for poisoned messages and persistent transport failures.
