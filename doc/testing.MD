# Courier Testing

## Conformance Suite Scope

- [ ] Golden scenarios: happy path, ack loss, sender crash before/after commit, receiver crash before/after ack, duplicate replay after dedup window.
- [ ] Invariant assertions: at-least-once delivery, no pre-commit visibility, one-attempt-per-message-per-tick, duplicate suppression by configured strategy.
- [ ] Language parity checks for wire format and state transitions (`Msg`, `Pact`, `Receipt`, events).
