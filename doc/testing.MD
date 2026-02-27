# Courier Testing

## Conformance Suite Scope

- [ ] Golden scenarios: happy path, ack loss, sender crash before/after commit, receiver crash before/after ack, duplicate replay after dedup window.
- [ ] Invariant assertions: at-least-once delivery, no pre-commit visibility, one-attempt-per-message-per-tick, duplicate suppression by configured strategy.
- [ ] Language parity checks for wire format and state transitions (`Msg`, `Pact`, `Receipt`, events).

## Shared Fixture Suite

Fixture path:

- `conformance/fixtures/suite.v1.json`

Current cross-language coverage:

- Message decode parity: missing `version` defaults to `1`.
- Tx validation parity: reject invalid `version=0`; reject non-ACK payloads containing ACK correlation fields.
- ACK apply parity: validate ACK correlation fields and apply recipient-list shrinking semantics.
- Network interop e2e topology matrix over TCP newline-JSON:
- `go -> typescript -> rust`
- `rust -> go -> typescript`
- `typescript -> rust -> go`
- ACK round-trip validation: terminal node must return `Ack` with valid correlation fields (`ack_msg_id`, `ack_from_id`, `ack_to_id`, `ack_version`).
- Multi-message sessions: each topology test sends multiple messages (`count > 1`) against a single server lifecycle.
- Negative live-network cases: malformed terminal ACK injection (`missing_ack_msg_id`, `bad_ack_version`) must be rejected by clients.

Runners:

- Rust: `rust/tests/conformance_fixture_tests.rs`
- Go: `golang/src/pkg/conformance` + `golang/tests/conformance_test.go`
- TypeScript: `typescript/src/pkg/conformance.ts` + `typescript/tests/conformance.test.ts`

Execution commands:

1. `cargo test -q`
2. `cd golang/src && go test ./...`
3. `cd golang/tests && go test ./...`
4. `cd typescript && npm test`
5. `./conformance/e2e/test_go_ts_rust.sh`
6. `./conformance/e2e/test_all_topologies.sh`
7. `./conformance/e2e/test_negative_cases.sh`
8. `./make.sh interop test`
