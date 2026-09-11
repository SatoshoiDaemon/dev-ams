# Environmental Systems

Regions may define environmental rules independently of enemies.

The Ice Mountain is the primary initial example.

Environmental systems should support concepts such as:

```text
temperature
weather
visibility
terrain
resource pressure
movement modifiers
regional status effects
```

These mechanics should be data/configurable where practical.

A region may enable a system and define parameters rather than requiring unique Rust logic for every map.

For example:

```text
environment:
    cold_intensity = 0.8
```

rather than:

```rust
if region == IceMountain {
    ...
}
```

when a generic environmental system can represent the behavior.
