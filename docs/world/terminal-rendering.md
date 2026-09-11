# Terminal Region Representation

World regions must remain compatible with terminal rendering.

Do not design maps that fundamentally require a graphical renderer.

Map data should be representable through structures such as:

```text
tiles
cells
rooms
nodes
coordinates
entities
connections
```

The terminal UI may render these using symbols, text, colors, borders, or other terminal capabilities.

However, world simulation should not depend on ANSI rendering details.

Keep:

```text
World Model
```

separate from:

```text
Terminal Representation
```
