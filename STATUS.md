# Status

## Current Position

- Project identity is now `trx`: a transmit/receive messaging foundation.
- Canonical base API is `Msg`, `Tx`, and `Rx`.
- Rust is the only implementation with a real runtime/reliable core in this repo.
- Go and TypeScript currently provide staged `trx` fixtures, conformance helpers, and interop scaffolding.
- Courier is now treated as a specialized reliable layer built on top of `trx`, not the base API itself.

## Landed

- Rust crate root exports `trx` as the canonical public base surface.
- Rust `Msg.body` now uses raw bytes in memory with base64 in the JSON wire envelope.
- Rust reliable code lives under `reliable` with `CourierMsg` as the framework-specific wrapper.
- Cross-language conformance fixtures exist for base `trx` message decoding/defaulting behavior.
- The legacy `rustie/msg` bridge has been removed from the `rustie` submodule.

## Still In Flight

- Shared durable implementations still need to move into `rustie`, `golangie`, and `tscriptie`.
- Non-Rust language roots in this repo still include migration-era scaffolding and legacy `msg` artifacts.
- Production transport adapters and production-safe storage backends are still incomplete.

## Working Rules

- Treat this repo as the architecture/spec/interop/conformance repo.
- Treat shared language submodules as the durable home for reusable library packages.
- Keep reliable-delivery semantics separate from the canonical outer `trx` envelope.

See [README.md](/Users/jkassis/Code/courier/README.md), [doc/ARCH.md](/Users/jkassis/Code/courier/doc/ARCH.md), and [doc/TODO.md](/Users/jkassis/Code/courier/doc/TODO.md) for the current migration plan.
