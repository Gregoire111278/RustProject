# 0003 - Use Thread Pool for Robot Concurrency

## Status
Proposed

## Context

As the number of robots in the simulation increases, the cost of sequentially processing each robot’s logic per tick becomes noticeable. Especially with more complex modules (e.g., scanning, smart movement, reporting), performance becomes a concern.

We considered multiple models for handling concurrency:
- Spawning a thread per robot (overhead, unsustainable for many robots)
- Keeping it sequential (simpler, but slower)
- Using async runtimes (requires significant refactor)
- Using a thread pool

## Decision

We propose using a **fixed-size thread pool** to distribute robot logic execution in parallel:
- A worker pool handles robot tick logic (movement, scanning, collection)
- Results are sent back to the main simulation loop for state updates
- We can use `rayon`, `crossbeam`, or a custom thread pool implementation

This parallelizes robot operations while avoiding the cost of spawning threads on each tick.

## Consequences

### Pros:
- Scalable execution model for large swarms
- Reuses threads efficiently
- Improves tick performance as robot count increases
- Easier to manage than per-thread robots

### Cons:
- Adds complexity around synchronization and shared state (e.g., map, logs)
- May require `Arc<Mutex<>>` patterns or message passing for safety
- Debugging becomes harder due to non-deterministic execution order

## Alternatives Considered

### 1. Per-thread-per-robot
- Simple in concept, but extremely heavy on system resources

### 2. Async (e.g., `tokio`, `async-std`)
- Very powerful, but would require refactoring large parts of the simulation engine
- Overhead not justified for simple per-tick logic

### 3. Keep it fully sequential
- Easier to test/debug
- Not scalable for large maps or many robots

## Related Decisions
- ADR 0001: Message passing with `mpsc`
- TBD: robot-to-robot interaction via messages vs shared state

## Notes
Thread pools provide a practical balance of performance and simplicity for this stage. Should the simulation grow in complexity (e.g., real-time rendering, hundreds of agents), migration to async may be reconsidered.
