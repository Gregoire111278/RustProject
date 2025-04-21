# 0001 - Use `mpsc` for Robot-Station Communication

## Status
Accepted

## Context

The project simulates an autonomous swarm of robots exploring a planet surface, collecting resources, and returning to a central station. The robots and station need to communicate efficiently in a concurrent environment.

We needed a reliable and simple method for message passing between threads, specifically for:
- Robots sending back reports to the station
- The station issuing commands (e.g., spawning new robots)

## Decision

We decided to use Rust’s standard library `std::sync::mpsc` channels to facilitate communication between the station and the rest of the simulation.

- The main thread spawns the `Station` in a separate thread.
- Two channels are used:
    - `tx_report`: robots send their reports to the station
    - `tx_cmd`: the station sends commands back to the simulation

## Consequences

### Pros:
- Simple to implement and understand
- Suitable for a low-complexity communication model
- Works well for single-producer/single-consumer or single-producer/multi-consumer
- No need for external dependencies

### Cons:
- No built-in support for async/await
- More difficult to scale to highly concurrent models if needed later

## Alternatives Considered

### 1. `tokio::mpsc`
- Rejected because the project does not currently use an async runtime
- Would require adding `tokio` and rewriting logic around async

### 2. Shared memory / `Arc<Mutex<T>`
- Considered too error-prone due to the risk of deadlocks
- Requires careful locking/unlocking logic and doesn’t fit the message-passing model well

## Related Decisions
- TBD: whether we later migrate to async or use worker pools if concurrency becomes more complex

## Notes
This decision is sufficient for the current scope. If robot behaviors or simulation size increases, we may revisit this in a future ADR.
