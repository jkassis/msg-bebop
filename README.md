# Courier

![Build](https://github.com/jkassis/courier/actions/workflows/test.yml/badge.svg)
📊 [View Coverage Dashboard](https://jkassis.github.io/courier/)

Reliable, multi-language messaging infrastructure with unified testing and coverage reporting.

Courier guarantees crash fault tolerant, exactly once, message delivery by leveraging a durable, transactional database and configurable retry logic.

Courier is implemented in a variety of languages to integrate systems with heterogeneous implementations.

Messages are stored in a database, and retried until acknowledged or timed out. The system supports inbound and outbound message queues, automatic cleanup of delivered messages, and robust error handling. Each message’s lifecycle is managed by a state machine, ensuring at-least-once delivery and consistent state across Rust, Go, and TypeScript components.

Courier’s communication is abstracted through the Tx and Rx interfaces (traits), allowing flexible message delivery. This design lets you implement synchronous in-process delivery (as shown in SyncTxRx for testing), or swap in other mechanisms—such as channels, threads, or network transport—without changing the core logic. By programming to these interfaces, Courier can support a variety of delivery strategies, from fast in-memory tests to real networked communication, all using the same message-handling code.

Courier allows clients to implement custom retry strategies through a `pact_factory` and a `pact_ticker`.

All implementations are continuously tested and integrated with full coverage reporting.

---
## Setup
To setup a Courier, you must provide...
- an implementation of `Courier::db::DB` so that Courier can manage message persistence. see the documentation on `Courier::tx`, `Courier::tick`, and `Courier::rx` to understand how Courier manages db transactions. In short, Courier relies you your application to create and commit db transactions during `tx` and `tick`, but manages its own 
- a `pact_factory` to yield initial Pacts during Courier::tx.
- a `pact_ticker` to control the retry strategy. The `pact_ticker` returns a Result<u64, String> with the target tick for the next send attempt or a String error if we should not retry. Courier takes care of updating `Pact.tick_of_last_attempt` when retrying.
- a `handler` function to handle inbound messages once received.
- a `sender` fn that performs the actual communication.

## Transaction Management



## Sequencing

### Tx Phase
Txer creates a new db txn
Txer calls Courier::tx
Courier::tx
 - retrieves the dbtx from the context
 - saves the message to the db
 - creates a pact to track retries and timeouts
 - saves the pact to the db
 - schedules an attempt to send on the next tick

Txer can continue its business logic. It must commit the db txn so that the courier database sees the message and the scheduled send_event. If it cancels the db txn, the message will not send. This allows the Txer to perform a transactional send of multiple messages.

### Tx Tick
On the next call to Courier::tick, Courier scans the database for scheduled outbound sends. For each message scheduled to send on or before that "tick", Courier::tick calls Courier::post.

If any call to Courier::post fails during the tick




```mermaid
sequenceDiagram
    participant Txer 
    participant CourierA as Courier A
    participant DBA as DB A
    participant Rxer 
    participant CourierB as Courier B
    participant DBB as DB B

    Txer: Create new DBTxn
    Txer->>CtxA: Store DBTxn 
    Txer->>CourierA: Send message with CtxA
    CourierA->>DBA: Store outbound message & pact
    CourierA->>CourierB: Send message
    CourierB->>DBB: Store inbound message
    CourierB->>CourierB: Process message (handler)
    CourierB-->>CourierA: Send Ack
    CourierA->>DBA: Delete outbound message & pact on Ack
    CourierB->>DBB: Delete inbound message on Ack
```

---

## Implementations 

### Rust
- 🦀 [Rust Docs](https://jkassis.github.io/courier/docs/rust/)
Uses SledDB for persistence.


### Typescript 
- 🧪 [TypeScript Docs](https://jkassis.github.io/courier/docs/ts/)


### GoLang
- 🐹 [Go Docs](https://pkg.go.dev/github.com/jkassis/courier)


---

## Coverage

<!-- COVERAGE-TABLE-START -->
<!-- COVERAGE-TABLE-START -->
## Coverage

| Language   | Coverage                      |
| ---------- | ----------------------------- |
| Rust       | ![](coverage-badges/Rust.svg) |
| Go         | ![](coverage-badges/Go.svg)   |
| TypeScript | ![](coverage-badges/TS.svg)   |
<!-- COVERAGE-TABLE-END -->
<!-- COVERAGE-TABLE-START -->
<!-- COVERAGE-TABLE-START -->
## Coverage

| Language   | Coverage                      |
| ---------- | ----------------------------- |
| Rust       | ![](coverage-badges/Rust.svg) |
| Go         | ![](coverage-badges/Go.svg)   |
| TypeScript | ![](coverage-badges/TS.svg)   |
<!-- COVERAGE-TABLE-END -->
