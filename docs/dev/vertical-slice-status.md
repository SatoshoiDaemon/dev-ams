# Combat Vertical Slice Status

> **Implementation status:** Playable vertical slice. This page describes what the
> current executable actually implements; it does not promote deferred game systems
> to completed features.

## Playable path

Run `ams` from an installation root containing `config.toml`, `data/`, and
`gamemodes/`. The terminal flow is:

```text
new demo base:standard
combat base:training-encounter
actions
use base:black-flame-orb-demo base:raider-1
end
save
explain last
```

The encounter contains one player-controlled Demon and two deterministic training
enemies. It demonstrates Physical and True Damage, Attribute Power and Scaling,
Shield, Tenacity, Block, Parry, Dark Flame, racial healing/buffs, Broken Heart,
Victory, Defeat, and Fled outcomes. The values belonging only to this encounter are
versioned demonstration content, not global racial defaults.

## Implemented

| Area | Current implementation |
| --- | --- |
| Combat state | Serializable `CombatSession` with closed rounds, explicit phases, declarations, stable Priority queue, pending Casts, AP, reactions, Bonus Action limits, event/command queues, racial state, outcomes, and RNG state. |
| Actions | Typed data definitions, atomic AP/Mana payment, declaration and resolution validation, `None`, `Self`, `Single`, `Multiple`, and resolution-time `All` targeting, ordered action steps, Scaling sources, and typed rejections. |
| Reactions | A single 50 AP Block or Parry reservation per actor. The reservation is consumed only when the corresponding reaction is attempted. Power/rating belongs to action data. |
| Events and death | Stable FIFO event processing with depth 16 and 256 events per action. HP mutation, triggers/Lua commands, and the orchestrated death confirmation are separate stages. |
| Statuses | The base catalog is loaded from `data/content/statuses.json`; competition, Resistance, ticks, decay, removal, immunity, and Dark Flame DoT priority use generic engine primitives. |
| Demon | Tag-driven Dread of Society, seven per-combat Hearts, exact half-HP crossing, Shield/debuff response, Dark Flame immunity, and the two racial spell templates. There is no `match Race::Demon`. |
| Mods and Lua | Deterministic discovery, dependency blocking, isolated load failures, one sandboxed Lua 5.4 runtime per mod, load-time registration, copied queries, queued `EngineCommand` mutations, namespaced listeners, stable listener order, and a configurable instruction budget. IO, OS, package, debug, native loading, network, and shell access are absent. |
| Persistence | Save format v2 ZIP with `metadata.json`, `ruleset.json`, `entities.json`, optional `combat.json`, and `mod_data.json`; active combat and `explain last` round-trip. V1 loads without combat, preserves entities/mod data, and reports the missing ruleset snapshot. |
| Configuration | Compiled safe defaults → `config.toml` → gamemode by internal ID → saved ruleset snapshot. `base:standard` ships in `gamemodes/standard.toml`. |
| Diagnostics | `latest.log`, timestamped session logs, severity levels, contextual fields, retention, panic reports under `logs/crashes/`, and normal/verbose combat output. No diagnostic is transmitted. |
| Terminal | Menu, ready, and combat commands over stdin/stdout. Invalid commands do not mutate simulation state. Unsaved replacement/quit requires confirmation; Ctrl+C offers the save-or-discard path. |
| Portability | Rust 1.80 CI on Windows/Linux, static `x86_64-unknown-linux-musl`, portable packaging scripts, and copied-artifact smoke tests. Lua is vendored. |

## Decisions closed by this milestone

- Combat uses closed rounds with explicit declaration and resolution phases. The
  player group receives only the first valid opening opportunity; later ordering is
  Priority, player group, then stable actor ID.
- Declaration validation precedes atomic payment. Accepted costs are not refunded by
  default, and actor, targets, and requirements are revalidated at resolution.
- Cast eligibility, interruption, Block/Parry reservation, Bonus Action insertion
  limits, immediate Flee, event limits, and post-trigger death confirmation now have
  executable contracts and regression tests.
- Attribute Power feeds declared Scaling sources; Power is not direct damage. Action
  steps, status outcomes, and Lua mutations re-enter engine primitives instead of
  bypassing the combat pipeline.
- Base content and the Demon demonstration use the same registries, events, data
  schemas, and controlled Lua-facing commands available to mods. Replacement of a
  stable ID must be explicit.
- Lua API v1, save format v2, configuration precedence, diagnostics, CLI commands,
  and the portable installed layout are defined and implemented for this slice.

## Deliberately partial inside the slice

- The typed `ActionRequirement` vocabulary covers active/tag/status/Mana/HP cases.
  Equipment, element ownership, and future subsystem validators are added when those
  systems become executable; actions do not contain bespoke terminal-side rules.
- The Glyph registry contains the canonical catalog and spell-template schemas use
  Nodes/Glyph IDs, but Glyphs requiring spatial/world/future systems do not yet have
  executable semantics.
- `UnresolvedContentReference` is the persistence representation for a missing live
  content reference. The vertical slice has no inventory/equipment live-reference
  owner yet, so no gameplay path currently creates one.
- The terminal renderer is intentionally line-oriented. Exploration and a map
  renderer are outside this slice.

## Future systems

Exploration, open world, Human, Angel, Elf, Feral, Dragonborn, full progression,
general inventory, crafting, economy, alchemy, companions, fast travel, and endgame
remain specifications or future work. Their documentation must not be read as an
implementation-status claim.

## Open work after this milestone

- The local Windows release artifact was built and launched from a copied directory.
  The static musl job and cross-platform packaging are configured in CI, but still
  need execution and artifact inspection on the remote CI runners.
- `ActionRequirement` must gain equipment, element, catalyst, learned-node, and
  Fighting Style validators when those owning systems become executable.
- Inventory/equipment will be the first live subsystem to consume
  `UnresolvedContentReference`; opaque missing-mod data is already preserved.
- The Glyph registry and spell-template contract are present, but executable
  semantics remain intentionally incomplete for Glyphs that depend on editing,
  spatial rules, exploration, equipment, or other future systems.
- Training enemies intentionally have one deterministic action. A reusable general
  AI system, broader encounter content, and balance work are not part of this slice.
- Synchronizing release documentation and binaries to the separate `ams` and
  `wiki-ams` repositories remains a release-management task, not an engine feature.

## Suggested next steps

1. Run the repository CI remotely, inspect the Windows and static-musl portable
   artifacts, and publish only artifacts that pass the copied-directory smoke test.
2. Publish modder-facing schema examples and a sample JSON/Lua mod that exercises
   registration, queries, events, queued commands, and isolated listener failure.
3. Implement equipment and inventory persistence/integration, extending the central
   requirement registry and unresolved-reference behavior without moving content
   rules into the terminal.
4. Connect equipment, catalysts, Fighting Styles, and the executable non-spatial
   Glyph subset to the existing declared Scaling and action-step pipeline.
5. Build the first exploration/map slice and its transition into the existing combat
   orchestrator; add spatial primitives only when that slice requires them.
6. Add the next playable race through data and events after its canonical contract is
   ready, while keeping all unimplemented races explicitly marked as future.
7. Harden compatibility with golden save/trace fixtures, further migrations, and
   property or fuzz tests for malformed content, configuration, saves, and Lua input.

## Verification

The regression suite covers the numeric foundation, damage pipeline, event limits,
status competition/ticks, action ordering and atomic validation, Demon boundaries,
reaction and Bonus Action limits, Lua isolation, mod dependency failures, gamemode
precedence, v1 migration, v2 active-combat round trips, deterministic Victory, and
Fled. CI additionally runs formatting, warning-free Clippy, release compilation,
static-musl verification, packaging, and launch from a copied artifact.
