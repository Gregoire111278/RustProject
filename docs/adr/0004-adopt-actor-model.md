# 0004 - Adopt Actor Model for Robot Concurrency

## Status
Accepted

## Context

Currently, robot logic (scanning, movement, collection, reporting) is executed sequentially in the main simulation loop. As the swarm grows, this sequential approach will become a performance bottleneck and complicate responsiveness (e.g., UI updates, station merges). We need a concurrency model that:

- Distributes per-robot computation across CPU cores
- Minimizes thread-creation overhead
- Keeps each robot's state ownership local to avoid contention
- Integrates cleanly with our existing `mpsc` channels between robots and station

## Decision

We will refactor the robot processing into an **actor model**: each robot runs in its own long-lived task that:

- Receives tick and control messages via an `mpsc::Receiver<RobotCmd>`
- Owns its `Robot` state internally, avoiding shared mutable state
- Processes messages (`Tick`, `Snapshot`, `Shutdown`) in a loop, updating its internal state and sending `RobotReport` back to the station over `mpsc::Sender<RobotReport>`
- Terminates cleanly on a `Shutdown` message

A coordinating component will broadcast `Tick` and other control messages to all robot actors each frame. The station remains unchanged in terms of receiving `RobotReport`s and merging diffs.

## Why Actor Model?

- **Isolation**: Each actor owns its state; no locks required
- **Scalability**: Actors map directly to OS threads or a small async runtime pool
- **Determinism**: Messages are processed sequentially per actor, preserving local logic order
- **Integration**: We already use `mpsc` for station communication; extending that to robot commands is consistent

## Consequences

### Pros
- Computes each robot’s behavior in parallel, leveraging all cores
- Eliminates the need for global locks on robot collections
- Simplifies reasoning about each robot’s internal state
- Actors can be paused, resumed, or replaced dynamically (e.g., respawn, reconfiguration)

### Cons
- Introduces boilerplate: channel setup, actor spawn, and teardown logic
- Debugging distributed actors can be more challenging
- Slightly more complex startup/shutdown sequence

## Alternatives Considered

### Thread Pool (ADR 0003)
- Good reuse of threads but still requires locking around shared `robots` vector

### Async Runtime
- Would require migrating all code to `async/.await`, heavier refactor

### Event Loop with Work Queue
- Centralized queue of tasks; closer to thread-pool but reintroduces shared queues

## Related Decisions
- ADR 0001: Use `mpsc` channels for station ↔ robot communication
- ADR 0003: Proposed thread-pool-based parallelism (rejected in favor of actor model)
