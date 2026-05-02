# trx

`trx` is a multi-language transmit/receive messaging foundation.

Its purpose is to standardize a minimal one-way messaging API across languages and transports:

- `Msg`
- `Tx`
- `Rx`

The base contract is intentionally simple. It should support:

- in-process sync transport for tests
- HTTP / WebSocket / gRPC / socket-based transports
- cross-language interoperability
- higher-level frameworks layered on top

## Core Idea

`trx` standardizes **one-way transmission**.

It does **not** try to force reliable messaging, ACK semantics, retries, durable inbox/outbox behavior, or synchronous wait-for-ack semantics into the universal base API.

Those belong to specialized frameworks built on top of `trx`.

## Canonical Message Shape

Canonical `Msg` is the stable outer envelope for one-way transmission.

Its fields are:

- `id`
- `from_id`
- `to_ids`
- `type_`
- `version`
- `body`

`body` is moving toward **binary / raw bytes** as the canonical payload representation. Structured payloads can be encoded on top by each framework or application.

## Layering

The intended architecture has three layers:

1. `trx` base API
   - `Msg`
   - `Tx`
   - `Rx`

2. transport adapters
   - HTTP
   - WebSockets
   - gRPC
   - custom sockets / UDP
   - in-process sync transport

3. specialized frameworks
   - example: reliable messaging

## Courier / reliable

The existing Courier work is being reframed as a specialized reliable messaging framework layered on top of `trx`.

Directionally:

- `msg` package becomes `trx`
- Courier-specific work moves toward a `reliable` package
- `CourierMsg` remains a framework-specific type, not part of the universal `trx` API

## Shared Library Submodules

The reusable implementation work should live in the language-specific shared submodules:

- Rust: `rustie`
- Go: `golangie` (planned)
- TypeScript: `tscriptie` (planned)

This repository is the coordination and verification repo for:

- architecture docs
- conformance fixtures
- end-to-end and interoperability tests
- cross-language migration planning

## Current State

- Rust has the only real implementation today.
- Go and TypeScript currently provide fixture/interoperability scaffolding.
- The repo still contains Courier-oriented naming and structure that will be migrated.

## Near-Term Work

- rename the project framing to `trx`
- rename `rustie/msg` to `rustie/trx`
- create `golangie` and `tscriptie`
- move reusable code into the shared submodules
- keep cross-language tests and e2e verification here
- move Courier-oriented reliable-delivery work toward a `reliable` package

See [doc/ARCH.md](/Users/jkassis/Code/courier/doc/ARCH.md) and [doc/TODO.md](/Users/jkassis/Code/courier/doc/TODO.md) for the current architecture and migration plan.
