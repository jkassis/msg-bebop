# AGENTS.md

## Project Identity

- The project is being reframed as **`trx`**: a multi-language transmit/receive messaging foundation.
- `Tx` / `Rx` are the architectural center.
- Canonical `Msg` is the stable one-way envelope.
- Higher-level messaging frameworks build on top of `trx`.
- `Courier` is no longer the product identity; it is one specialized framework layered on top of `trx`.

## Repository Role

This repository is primarily for:

- cross-language architecture and specification
- end-to-end and interoperability testing
- shared fixtures and conformance scenarios
- migration planning across languages

The reusable implementation work should live in the language-specific shared library submodules:

- Rust: `rustie`
- Go: `golangie` (to be created)
- TypeScript: `tscriptie` (to be created)

This repo may temporarily contain migration code, adapters, or staging implementations, but the intended durable home for reusable library code is the shared submodules.

## Package Direction

- `rustie/msg` should be renamed to `rustie/trx`.
- Equivalent `trx` packages should be created in `golangie` and `tscriptie`.
- Courier-specific reliable messaging should move toward a `reliable` package rather than remaining the conceptual center of the repo.

Target shape:

- `trx`: canonical one-way messaging API
- `reliable`: specialized reliable messaging framework built on `trx`

## Canonical API

The base `trx` API should stay simple:

- `Msg`
- `Tx`
- `Rx`

`Msg` contains only fields needed for one-way transmission:

- `id`
- `from_id`
- `to_ids`
- `type_`
- `version`
- `body`

`body` should shift toward **binary / raw bytes** as the canonical payload representation, with language-specific helpers for encoding/decoding structured payloads on top.

## Specialization Model

- Do not extend the canonical outer `Msg` envelope with framework-specific ACK/reliability fields.
- Message specialization should happen through:
  - `type_`
  - `body`
  - framework-specific wrappers when justified

- `CourierMsg` is allowed as a Courier-specific wrapper/snowflake.
- Reliable-delivery semantics are not expected to standardize as cleanly as the base `trx` API.

## Boundary Rules

- Transport concerns belong behind `Tx` / `Rx`.
- Edge authentication terminates at ingress.
- Internal `from_id` trust is a deployment/framework boundary decision, not a base `trx` feature.
- Retry, ACK, dedup, durable inbox/outbox, expiration, and synchronous wait-for-ack semantics belong to higher-level frameworks such as `reliable` / Courier, not to the canonical `trx` surface.

## Docs Expectations

When updating docs, keep these distinctions explicit:

- `trx` is the shared messaging foundation
- shared library code belongs in `rustie` / `golangie` / `tscriptie`
- this repo hosts cross-language tests, fixtures, e2e topology tests, and architectural coordination
- Courier is one specialized reliable framework, not the universal base API

## Migration Bias

When proposing work, bias toward:

1. clarifying `trx` semantics
2. moving reusable code into shared submodules
3. keeping this repo focused on spec, fixtures, and cross-language verification
4. treating reliable messaging as a separate framework layer
