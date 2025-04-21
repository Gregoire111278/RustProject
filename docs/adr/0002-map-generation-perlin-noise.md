# 0002 - Map Generation Using Perlin Noise

## Status
Accepted

## Context

The simulation requires a map that represents the terrain of a celestial body (e.g., planet surface). This map needs to include:
- Obstacles
- Resource tiles (Energy, Mineral, Science)
- Empty traversable tiles

The map must also be:
- Procedurally generated
- Reproducible from a seed
- Configurable in size

We wanted a natural-looking terrain that avoids pure randomness and provides clusters or patterns for better exploration and strategic movement.

## Decision

We chose to use **Perlin noise** via the `noise` crate to generate the map.
- Each tile is assigned a value from the Perlin noise function.
- Based on thresholds, tiles are classified into obstacles, resources, or empty tiles.
- A `StdRng` seeded from a fixed or dynamic seed ensures deterministic generation.

## Consequences

### Pros:
- Natural and organic terrain distribution
- Easy to tweak thresholds to tune the balance of obstacles and resources
- Reproducibility from a fixed seed supports debugging and repeat experiments
- Lightweight and performant

### Cons:
- May need tuning for different map sizes to avoid bias toward certain tile types
- Requires learning curve to understand noise parameters and effects

## Alternatives Considered

### 1. Pure randomness (e.g., `rand::Rng` only)
- Rejected due to overly chaotic and unrealistic terrain generation

### 2. Predefined static maps
- Rejected to maintain replayability and procedural generation requirements

### 3. Other noise functions (Simplex, OpenSimplex)
- Could be used in future for different effects
- Perlin noise was sufficient and easy to implement

## Related Decisions
- TBD: whether to implement different map types (e.g., spherical wrapping, biomes)

## Notes
This approach provides a good foundation for map diversity and consistent robot testing environments. The tile classification thresholds are currently hardcoded but can be moved to config files in future enhancements.
