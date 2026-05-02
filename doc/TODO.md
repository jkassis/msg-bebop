# trx TODO

## Done

- Rust base message abstraction exists: `Msg`, `Tx`, and `Rx` in `rust/src/rustie/trx/`.
- Staged `trx` packages now exist in Rust, Go, and TypeScript in this repo as a migration bridge toward the shared language submodules.
- `SyncTx` exists as an in-process transport for tests and local composition.
- Rust core exists for `tx`, `tick`, `rx`, message/pact/receipt models, receipt-based idempotency, expiration hooks, and observability recorder plumbing.
- Rust ACK handling validates correlation fields and supports multi-recipient progress by shrinking `to_ids` as recipients ack.
- Cross-language fixture parity exists for message version defaulting, tx validation, and ACK-apply behavior.
- Live interop scaffolding exists across Go, TypeScript, and Rust over newline-delimited JSON with positive, negative, retry-recovery, and relay-fault scenarios.
- Architecture direction is now decided: canonical one-way `Msg` remains the standard API, and Courier will use a Courier-specific wrapper named `CourierMsg`.
- Project direction is now decided: `trx` is the primary product identity, and Courier is a framework layered on top of `trx`.
- `Msg.body` direction is now decided: shift toward binary / raw bytes as the canonical payload representation.
- A concrete cross-language wire contract now exists for base `trx` messages: in-memory raw bytes, JSON envelope body encoded as base64.
- Shared `trx` conformance fixtures now exist for Rust, Go, and TypeScript.
- A first concrete Rust `reliable` module now exists, including `CourierMsg` plus bridge conversions to the current legacy Courier wire shape.
- The Rust crate root now treats `trx` as the canonical public `Msg` / `Tx` / `Rx` surface, while Courier-specific wire semantics live under `reliable`.

## To Production Readiness

### 1. Define and stabilize the base messaging component contract

- [ ] Rename the project, docs, package descriptions, and release framing from `courier` to `trx`, while preserving Courier as the name of the reliable-delivery framework.
- [ ] Promote `Msg` + `Tx` + `Rx` to the documented primary API of the project across Rust, Go, and TypeScript.
- [ ] Specify the minimal guarantees of the base API separately from any higher-level guarantee layer.
- [ ] Document a component model with three layers: base API, transport adapters, and guarantee layers such as Courier.
- [ ] Define how wrappers/composition are expected to work around `Tx` / `Rx` so that reliability, observability, validation, and auth can be layered cleanly.
- [ ] Lock the canonical `Msg` field set and explicitly remove Courier-specific ACK/reliability semantics from the conceptual base envelope.
- [ ] Clarify that specialization happens primarily through `Msg.body` schema plus `type_`, not by extending the canonical outer envelope.
- [ ] Change the conceptual and implementation target for `Msg.body` to binary / raw bytes across languages.
- [ ] Define the canonical wire representation for raw `Msg.body` in Rust, Go, and TypeScript.
- [ ] Define standard codec helper conventions above raw `body` for JSON and other structured payloads.

### 2. Move reusable implementation work into shared submodules

- [x] Rename `rustie/msg` to `rustie/trx`.
- [ ] Create the `golangie` shared submodule.
- [ ] Create the `tscriptie` shared submodule.
- [ ] Create `trx` packages in each shared language submodule.
- [ ] Move reusable base messaging code from this repo into the shared `trx` packages.
- [ ] Leave this repo focused on docs, fixtures, e2e tests, interop tests, and cross-language verification.
- [ ] Perform repo cleanup after migration checkpoints, including deleting temporary backup branches created during the `main`/`origin/main` reconciliation once they are no longer needed.

### 3. Define trust boundaries and security posture

- [ ] Document the edge-authentication model explicitly: authentication terminates at ingress, then normalized internal messages carry trusted `from_id`.
- [ ] Define the threat model for “trusted internal sender ids” and the deployment assumptions required to make that safe.
- [ ] Decide whether any proof-of-auth metadata is ever part of the base framework, or always an application-level wrapper concern.
- [ ] Define ACK authenticity expectations for reliable guarantee layers such as Courier.

### 4. Reconcile Courier as one guarantee-providing component

- [ ] Reframe Courier in docs and package structure as a higher-level reliability component built on the base message API.
- [ ] Introduce and document `CourierMsg` as the Courier-specific wrapper around canonical `Msg`.
- [ ] Decide how `CourierMsg` relates to `Msg.body`: wrapper-only, specialized Courier body payloads, or a hybrid.
- [ ] Decide which reliability metadata stays on `CourierMsg` versus in Courier-managed secondary state, preserving current Courier optimizations where they are intentional.
- [ ] Define the Courier-specific API surface, including how callers provide reliable-delivery options and whether/when they can synchronously wait for ACK completion.
- [ ] Move Courier-oriented code toward a `reliable` package built on `trx`.
- [ ] Decide whether Courier inbound handling remains the current **durable inbox** model or changes to an **inline handler** model, then align API, docs, events, and tests to that decision.
- [ ] Rename misleading Rust observability events to match the actual inbound model (`courier.rx.handler_success` -> `courier.rx.persisted`, etc.).
- [ ] Tighten the `tick()` contract in docs and code so transaction ownership is explicit and consistent.
- [ ] Remove or quarantine stale doc claims that imply multi-language feature parity where only Rust core exists.

### 5. Build real transport adapters behind the standard API

- [ ] Define transport-adapter responsibilities versus Courier responsibilities.
- [ ] Choose the first production transport(s) to implement behind `Tx` / `Rx` in Rust.
- [ ] Implement at least one network transport adapter in Rust with clear edge-auth hooks.
- [ ] Define transport capability differences: request/response, streaming, max payload size, ordering expectations, timeout behavior, and connection lifecycle.
- [ ] Decide whether transport adapters are pure message pipes or may also offer optional middleware chains.
- [ ] Define how transport adapters expose body serialization/deserialization hooks without breaking cross-language wire compatibility.

### 6. Make the Rust reliable storage path production-safe

- [ ] Replace or supplement `SledDB` with a backend that actually provides the transactional semantics Courier’s outbox contract requires.
- [ ] Prove rollback behavior with tests that fail mid-transaction and confirm no partial Courier state is committed.
- [ ] Define which backend(s) are supported for production and document their transactional and durability properties.
- [ ] Revisit key layout and scan behavior under load, especially `te:{tick}:...` and receipt cleanup scans.

### 7. Harden the Rust runtime and operator surface

- [ ] Add backpressure controls to `tick()`: bounded batch size, bounded time budget, and clear behavior on large tick jumps.
- [ ] Define single-worker vs multi-worker tick ownership; if multi-worker is desired, add a lease/fencing design.
- [ ] Publish an observability baseline that includes backlog depth, retry counts, ack latency, dedup hits, expirations, and inbox depth.
- [ ] Add runbooks for backlog spikes, poison messages, DB pressure, stuck retries, and ack-loss incidents.

### 8. Close the testing gap that still blocks go-live

- [ ] Add base `trx` conformance cases for raw-bytes `Msg.body` and canonical one-way transmission semantics.
- [ ] Add the missing golden scenarios from `doc/testing.MD`: sender crash before/after commit, receiver crash before/after ack, ack loss, and duplicate replay after dedup expiry.
- [ ] Add invariant tests for no pre-commit visibility, one-attempt-per-message-per-tick, and correct inbox/ack ordering.
- [ ] Add deterministic replay tests with fixed ticks, deterministic IDs, and deterministic retry policy functions.
- [ ] Add tests that exercise expiration hooks, DLQ flow, and receipt-window misconfiguration.
- [ ] Add load-oriented tests for bursty tick advancement and large send-event scans.
- [ ] Add transport-adapter conformance tests that can be reused across multiple transports in one language.

### 9. Decide the v1 scope honestly

- [ ] Either narrow the project claim to **Rust-first** for v1, or implement actual Go and TypeScript Courier cores.
- [ ] If Rust-first, make that explicit in the docs and release plan.
- [ ] Decide whether v1 promises only the base `Msg` / `Tx` / `Rx` API across languages, or Courier-level reliability APIs across languages.
- [ ] If multi-language remains a v1 goal, implement Go and TypeScript cores before claiming semantic parity.

### 10. Reach true multi-language parity if that remains in scope

- [ ] Add the base `Msg`, `Tx`, and `Rx` abstractions to Go and TypeScript with compatible semantics.
- [ ] Add `Msg.version` to all language models and ensure deserialization defaults to `1` when missing. (Rust done; Go/TS core pending)
- [ ] Build at least one transport adapter per language against that base API.
- [ ] Standardize `Msg.body` wire-format conventions so specialized payloads deserialize predictably across Rust, Go, and TypeScript.
- [ ] Create `golangie/trx` and `tscriptie/trx` as the long-term homes of those implementations.
- [ ] Add `CourierMsg` and Courier-specific APIs to Go and TypeScript only if Courier itself is part of the multi-language v1 scope.
- [ ] Add ACK validation fields (`msg_id`, `from_id`, `to_id`, `version`) and reject malformed/mismatched ACKs in the actual Go/TS Courier cores. (Rust done; Go/TS core pending)
- [ ] Add retry/idempotency guardrails, including receipt strategy validation (`keep_receipt >= max_retry_horizon` when statically knowable), in Go/TS cores. (Rust done; Go/TS core pending)
- [ ] Define typed error categories (`retryable`, `terminal`, `caller_bug`) consistently across Rust/Go/TS implementations. (Rust type exists; Go/TS core pending)

### 11. Repo cleanup

- [ ] Delete temporary local backup branches created during the unrelated-history sync once confidence is high that `main` contains everything needed.
- [ ] Delete stale feature branches after their work is fully landed on `main`.
- [ ] Remove obsolete files and examples that still reflect the old `msg`/Courier-centric project identity.
