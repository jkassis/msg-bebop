# trx / reliable Testing

## Conformance Suite Scope

- [ ] Golden scenarios: happy path, ack loss, sender crash before/after commit, receiver crash before/after ack, duplicate replay after dedup window.
- [ ] Invariant assertions: at-least-once delivery, no pre-commit visibility, one-attempt-per-message-per-tick, duplicate suppression by configured strategy.
- [ ] Language parity checks for wire format and state transitions (`Msg`, `Pact`, `Receipt`, events).
- [ ] Base `trx` parity checks for canonical `Msg`, raw-bytes `body`, and `Tx` / `Rx` behavior across Rust, Go, and TypeScript.

Current reality:

- Rust is the only implementation with a Courier/reliable core.
- Go and TypeScript currently provide fixture/interoperability scaffolding, not full Courier implementations.
- Existing interop tests validate envelope shape and ACK behavior across demo binaries; they are not equivalent to full cross-language `trx` + reliable conformance.
- This repository is the intended home for e2e tests, interop tests, and cross-language conformance, even as reusable implementation code moves into shared submodules.

## Shared Fixture Suite

Fixture path:

- `conformance/fixtures/suite.v1.json`

Current cross-language coverage:

- Message decode parity: missing `version` defaults to `1`.
- Message envelope parity for canonical `Msg`.
- Base `trx` body parity: raw bytes in memory, base64 in JSON wire envelopes.
- Tx validation parity: reject invalid `version=0`; reject non-ACK payloads containing ACK correlation fields.
- ACK apply parity: validate ACK correlation fields and apply recipient-list shrinking semantics.
- Network interop e2e topology matrix over TCP newline-JSON:
- `go -> typescript -> rust`
- `rust -> go -> typescript`
- `typescript -> rust -> go`
- ACK round-trip validation: terminal node must return `Ack` with valid correlation fields (`ack_msg_id`, `ack_from_id`, `ack_to_id`, `ack_version`).
- Multi-message sessions: each topology test sends multiple messages (`count > 1`) against a single server lifecycle.
- Retry recovery sessions: topology runs with terminal `drop-first` simulation and client retries enabled must still converge.
- Retry recovery sessions: topology runs with relay `drop-first` simulation and client retries enabled must still converge.
- Relay fault sessions:
- `drop_first_forward`: relay drops first forwarded request to simulate transient forwarding loss.
- `delay_ms`: relay injects per-request handling delay to exercise timeout budgets.
- Negative live-network cases: malformed terminal ACK injection (`missing_ack_msg_id`, `bad_ack_version`) must be rejected by clients.
- Runner uses active port health probes before client send (not fixed startup sleeps).

Runners:

- Rust: `rust/tests/conformance_fixture_tests.rs`
- Rust (`trx`): `rust/tests/trx_conformance_fixture_tests.rs`
- Go: `golang/src/pkg/conformance` + `golang/tests/conformance_test.go`
- Go (`trx`): `golang/src/pkg/trx/conformance_test.go`
- TypeScript: `typescript/src/pkg/conformance.ts` + `typescript/tests/conformance.test.ts`
- TypeScript (`trx`): `typescript/src/pkg/trx_fixture.test.ts`

Directionally, reusable `trx` implementation code should move into:

- `rustie/trx`
- `golangie/trx`
- `tscriptie/trx`

while this repository continues to host shared test fixtures and cross-language verification.

Execution commands:

1. `cargo test -q`
2. `cd golang/src && go test ./...`
3. `cd golang/tests && go test ./...`
4. `cd typescript && npm test`
5. `./conformance/e2e/test_go_ts_rust.sh`
6. `./conformance/e2e/test_all_topologies.sh`
7. `./conformance/e2e/test_negative_cases.sh`
8. `./conformance/e2e/test_retry_recovery.sh`
9. `./conformance/e2e/test_relay_faults.sh`
10. `./make.sh interop test`

## Interop Logs

- Per-topology logs are written to `conformance/e2e/logs/`:
- `*.client.log`
- `*.mid.log`
- `*.terminal.log`
- `*.summary.json`

In CI, these logs are uploaded as `interop-logs` artifacts on both pass/fail runs.
