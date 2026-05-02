# Publishing Guide

This repository is now the `trx` coordination repo, not the long-term home for reusable language packages.

Current publishing stance:

- Rust reusable code should move into the `rustie` shared submodule.
- Go reusable code should move into `golangie/trx`.
- TypeScript reusable code should move into `tscriptie/trx`.
- This repo should primarily publish docs, fixtures, conformance scenarios, and interop/e2e validation assets.

## Current State

- The Rust crate in this repo is now named `trx`.
- The Go, TypeScript, Python, C#, and C++ directories still contain migration-era scaffolding and legacy generated artifacts.
- Legacy `msg` package names and schema paths may still exist in staging code and tooling; do not treat them as the target public package identity.

## Before Publishing Anything

1. Decide whether the artifact belongs in this repo or in a shared language submodule.
2. Verify the artifact exposes the canonical `trx` base API (`Msg`, `Tx`, `Rx`) rather than legacy `msg` framing.
3. Run conformance and interop validation from this repo before publishing.
4. Ensure any reliable/Courier-specific surface is published as a higher-level package layered on `trx`, not as the base API itself.

## Rust

For now, the in-repo Rust crate can be published only if you intentionally want a migration-era package snapshot.

```bash
cd rust
cargo test
cargo publish
```

Target direction:

- base `trx` package should live in the `rustie` shared library submodule
- Courier/reliable package should be published separately from the base `trx` contract

## Other Languages

Do not publish the current in-repo Go/TypeScript/Python/C#/C++ staging packages as the canonical `trx` libraries without first moving them into their shared submodules and aligning naming, docs, and wire-contract behavior.

## Versioning

When versioning the migration bridge in this repo:

1. Keep the canonical wire contract stable across fixtures and interop tests.
2. Update Rust crate/package metadata and any staged language package metadata together when they intentionally ship as one release.
3. Keep schema/tooling path changes explicit because some scripts still reference `schema/msg.bop`.

## Release Rule

If an artifact is meant to be durable reusable library code, publish it from the language-specific shared submodule, not from this coordination repo.
