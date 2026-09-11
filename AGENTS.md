# AGENTS.md

Game name: A Magic Sovereign

Events:
OnAttackReceived
OnDamageReceived
OnHPDamage
OnTenacityDamage
OnShieldDamage
OnHPBelow

Explain last:
Feature that shows the exact initial calculation chain from the last turn.

The game needs builds that run on Windows and Linux.
No local server, Electron, browser, remote API, or mandatory launcher. The game opens directly in the terminal.

For Linux, the key requirement is to avoid dependencies on the player's system beyond the kernel and terminal. Use static musl builds, especially x86_64-unknown-linux-musl.

This comes close to supporting any Linux distribution on a compatible architecture.

License: MIT

Directories:
/dev-ams - Developer driven repository
/ams - Production and public repository
/wiki-ams - Wiki for modders and game systems

## 1. Project Philosophy

This project is an open-source, offline-first, terminal-rendered RPG written in Rust.

The game must remain fully functional without Internet access and must prioritize player ownership, modding, portability, transparency, and data-driven design.

The following principles are architectural requirements, not suggestions:

* No DRM.
* No authentication.
* No accounts.
* No cloud dependency.
* No telemetry or analytics.
* No Internet requirement.
* Do not add network dependencies unless the project's architecture is explicitly changed by the maintainer.
* Do not make saves, configuration, mods, or game data intentionally opaque.
* Player-owned data must remain inspectable and editable.
* Prefer explicit, deterministic systems over hidden behavior.
* Prefer data-driven behavior over hardcoded content.
* Modding is a first-class feature, not an afterthought.
* The base game should use the same systems exposed to mods whenever practical.

The final installed game must be portable.

A player must be able to copy the installed game directory to a USB drive, move it to another compatible computer with no Internet connection, and run the game without downloading additional runtimes, assets, Lua installations, authentication data, or online dependencies.

---

# 2. Technology

Primary language:

```text
Rust
```

Rendering:

```text
Terminal / CLI
```

Lua scripting must be embedded into the executable.

The user MUST NOT be required to separately install Lua.

When choosing Rust dependencies, prefer libraries that:

* work entirely offline at runtime;
* can be distributed with the game;
* do not require external services;
* do not introduce telemetry;
* do not require runtime package managers;
* work with portable/local filesystem paths.

Do not introduce Electron, browser rendering, web servers, cloud APIs, remote databases, or similar infrastructure to implement features that can be implemented locally.

---

# 3. Game Structure

The game has two primary interaction modes.

## Exploration

Exploration uses the terminal as the graphical interface.

The player moves through the world using CLI/terminal rendering.

The terminal is effectively the game's rendering engine.

Movement, map representation, entities, objects, enemies, interactables, and relevant environmental information should be representable through terminal output.

## Combat

Touching/engaging an enemy transitions the game into a turn-based combat engine.

Combat is primarily textual.

The combat engine should clearly describe:

* actions;
* attacks;
* damage;
* effects;
* status changes;
* resource changes;
* triggers;
* deaths;
* relevant calculations.

Combat logic MUST NOT depend on terminal rendering.

Keep simulation/state separate from presentation.

The terminal UI consumes game state and combat events; it should not itself define game rules.

The player can change the normal combat to a verbose combat, log.

---

# 4. Recommended Architectural Separation

Prefer separation similar to:

```text
Game State
    ↓
Game Systems
    ↓
Events / Results
    ↓
Terminal Presentation
```

Game rules should live in reusable systems/modules rather than inside UI code.

Examples:

```text
src/
├── main.rs
├── app/
├── terminal/
├── world/
├── combat/
├── entities/
├── attributes/
├── effects/
├── equipment/
├── magic/
├── glyphs/
├── items/
├── loot/
├── saves/
├── config/
├── gamemodes/
├── modding/
├── lua/
└── data/
```

This is a guideline rather than a mandatory exact directory structure.

Preserve clear boundaries between:

```text
simulation
content/data
persistence
modding
Lua API
terminal presentation
```

---

# 5. Filesystem Layout

The installed game should use a human-readable filesystem layout.

Expected root structure:

```text
/game
├── game executable
├── config.toml
├── saves/
├── gamemodes/
├── mods/
└── data/
```

Additional folders may be introduced when justified.

Do not hide essential player data in platform-specific application directories unless absolutely necessary.

Prefer paths relative to the game executable/root directory.

This is necessary for portability.

---

# 6. Saves

All saves belong in:

```text
/saves
```

A save is distributed/stored as:

```text
.zip
```

The contents of the save archive must use human-readable formats, primarily JSON.

Example:

```text
/saves/
    character_name.zip
```

Possible archive structure:

```text
character.json
world.json
inventory.json
quests.json
effects.json
metadata.json
```

Exact separation may evolve.

The following rule is mandatory:

> Players own their saves.

Players must be able to:

* open saves;
* extract saves;
* inspect their contents;
* edit JSON manually;
* recompress them;
* copy them;
* share them;
* back them up;
* restore them.

Do not encrypt saves.

Do not sign saves to prevent modification.

Do not implement anti-cheat for local save modification.

Do not intentionally obfuscate save data.

Do not reject a save merely because the player manually modified legitimate values.

Validation exists to prevent crashes and invalid state, not to police player behavior.

When loading malformed or partially invalid saves, prefer useful validation errors over panics.

---

# 7. Global Configuration

The main global configuration file is:

```text
/config.toml
```

This defines default/global rules applied to saves unless overridden through the appropriate game mode/preset system.

It should expose gameplay configuration whenever practical.

Examples include:

```text
diminishing returns
attribute soft caps
enemy HP multipliers
enemy damage multipliers
glyph weights
loot tables
hardcore rules
spawn rules
combat parameters
resource parameters
progression parameters
```

Do not hardcode values that reasonably belong in configuration or game data.

Defaults may exist in Rust as a fallback, but configurable gameplay values should normally be represented externally.

Configuration parsing must provide useful errors.

Unknown or invalid configuration should not cause unexplained crashes.

---

# 8. Game Modes / Presets

Game mode presets belong in:

```text
/gamemodes
```

Game modes represent reusable configurations that can be loaded for individual saves.

A save should be able to reference/use a specific preset without changing the global configuration for every other save.

Conceptually:

```text
config.toml
        ↓
global defaults

gamemodes/*.toml
        ↓
preset / ruleset

save
        ↓
selected game mode
```

Keep the exact precedence rules explicit and deterministic.

If configuration layers override one another, document their priority.

Avoid implicit configuration behavior.

---

# 9. Modding

The game is MOD-FIRST.

Mod support is a core architectural constraint.

The game must ship with:

```text
/mods
```

even when the directory is empty.

Mods should be capable of providing content through data files and behavior through Lua.

At minimum, the architecture should support modding for:

```text
glyphs
enemies
items
effects
passives
loot
combat behavior
magic
world content
```

The architecture should also be capable of supporting substantially larger Lua modifications, including custom subsystems.

Do not assume mods are limited to cosmetic or numerical changes.

---

# 10. Data-Driven Content

Prefer JSON, TOML, or another documented human-readable format for declarative content.

For example, a glyph should preferably be describable as data rather than requiring a Rust match statement for every glyph:

```json
{
  "id": "example.fire_power",
  "name": "Fire Power",
  "weight": 3,
  "effects": []
}
```

This example is illustrative and does not define the final schema.

IDs should be stable and namespace-friendly.

Prefer identifiers such as:

```text
base:fire
base:strength
example_mod:firestorm
author_mod:enemy_name
```

over indexes tied to load order.

Avoid architectures such as:

```rust
match glyph {
    Glyph::Fire => ...
    Glyph::Ice => ...
    Glyph::Blood => ...
}
```

when the same behavior can reasonably be represented by data, generic effects, registries, or Lua.

Hardcoding is acceptable for engine primitives.

Hardcoding content should be avoided.

---

# 11. Lua

Lua must be embedded in the game executable.

Installing Lua separately must never be required.

Lua is intended for behavior that cannot or should not be expressed through static data alone.

Mods should eventually be able to use Lua for things such as:

```text
passives
effects
triggers
enemies
AI
glyph behavior
combat mechanics
custom rules
custom subsystems
```

Expose an explicit, versioned modding API.

Do not expose arbitrary Rust internals directly.

Prefer controlled interfaces such as:

```text
game.register_effect(...)
game.register_glyph(...)
game.register_enemy(...)
game.on(...)
combat.damage(...)
combat.heal(...)
entity.get_attribute(...)
entity.add_effect(...)
```

Names above are conceptual examples, not mandatory API names.

Lua APIs should have clear contracts.

The modding API should eventually expose its own version, for example:

```text
mod_api_version = 1
```

Breaking API changes should be deliberate.

---

# 12. Mod Safety and Failure Isolation

A broken mod should produce a useful error whenever possible instead of crashing the entire game.

Errors should identify:

```text
mod
file
Lua script/data file
relevant object
reason
```

Example concept:

```text
Failed to load mod "example_mod":
mods/example_mod/scripts/fire.lua:42
attempt to access invalid entity
```

Do not silently swallow mod errors.

Do not allow one malformed JSON entry to produce an unexplained panic.

---

# 13. Registries

Prefer registries for extensible content.

Conceptually:

```text
GlyphRegistry
EnemyRegistry
ItemRegistry
EffectRegistry
DamageTypeRegistry
PassiveRegistry
LootTableRegistry
```

Base-game content and modded content should ideally enter the engine through the same registration/loading pipeline.

The base game should behave as much like a first-party mod as practical.

This minimizes privileged hardcoded content and continuously exercises the modding architecture.

---

# 14. Determinism

Game calculations should be deterministic wherever practical.

Randomness must pass through an explicit RNG system rather than scattered ad-hoc calls.

This is important for:

```text
debugging
testing
reproducibility
mods
save compatibility
combat verification
```

Allow deterministic seeded execution in tests.

---

# 15. Combat Damage Pipeline

Damage MUST be processed according to the following conceptual pipeline:

```text
BASE
 ↓
Equipment
 ↓
Active Effects
 ↓
Attributes
 ↓
Offensive Modifiers
 ↓
Defensive Modifiers
 ↓
Damage Type
 ↓
Shield
 ↓
Tenacity
 ↓
HP
 ↓
On Damage Triggers
 ↓
On HP Change Triggers
 ↓
Death Check
```

Do not casually reorder these stages.

Changes to pipeline order are gameplay changes and must be treated as such.

Each stage should be independently understandable and preferably testable.

Avoid implementing damage as one enormous function.

Prefer a damage context/pipeline architecture where each stage transforms or consumes a well-defined combat context.

Conceptually:

```text
DamageRequest
      ↓
DamageContext
      ↓
pipeline stages
      ↓
DamageResult
      ↓
combat events
```

A final damage result should preserve enough information for debugging, combat logs, tests, and mods to understand how the result was obtained.

---

# 16. Damage Types

Core damage types:

```text
Physical
Magical
True
```

Display names (English):

```text
Physical
Magical
True
```

True Damage ignores Tenacity.

Do not assume this means True Damage automatically ignores every other defensive mechanic.

Its explicit core property is:

```text
True Damage → ignores Tenacity
```

Other interactions should be controlled by their respective rules/configuration.

---

# 17. Fractional Results

Unless a system explicitly defines otherwise:

```text
fractional result → round down
```

Equivalent mathematical behavior:

```text
floor(result)
```

Do not use conventional rounding.

Examples:

```text
10.1 → 10
10.5 → 10
10.9 → 10
```

Centralize this behavior where practical to avoid inconsistent calculations.

---

# 18. Attribute Enhancement Source Limit

The same attribute may be enhanced by at most:

```text
4 sources
```

This limit applies regardless of how many effects are currently available.

Do not interpret this as four effects globally.

It is a per-attribute enhancement-source limit.

The system should retain enough source information to determine where each modification came from.

---

# 19. Competing Effects

When two effects of the same applicable/competing category are applied to the same target, compare:

```text
1. Potency
2. Counter
```

Potency has priority.

Counter is the tie-breaker.

The weaker effect is discarded.

Conceptually:

```text
if A.potency > B.potency:
    keep A

else if B.potency > A.potency:
    keep B

else:
    compare counter
```

Do not stack effects that are defined as mutually competing merely because they came from different sources.

The exact definition of effect identity/category should be explicit in the effect data model.

---

# 20. Attributes

Core attributes are:

```text
Vigor
Resistance
Strength
Dexterity
Precision
Intelligence
Mana
```

Display names (English):

```text
Vigor
Resistance
Strength
Dexterity
Precision
Intelligence
Mana
```

Keep attribute calculations centralized.

Do not scatter attribute-specific formulas throughout unrelated systems.

---

# 21. Vigor

Vigor increases HP.

Base gain:

```text
1 Vigor = +10 HP
```

Before configurable modifiers, soft caps, diminishing returns, or other systems are applied.

---

# 22. Resistance

Resistance increases Tenacity.

Base gain:

```text
1 Resistance = +10 Tenacity
```

Resistance/Tenacity also participates in debuff-duration reduction.

The exact diminishing-return curve for debuff reduction should be configurable rather than permanently buried in Rust code.

---

# 23. Mana

Mana increases the MP/mana resource pool.

Base gain:

```text
1 Mana = +10 MP
```

---

# 24. Strength

Strength represents physical power.

It affects:

```text
physical strength
unarmed damage
Strength-scaling weapon damage
physical attack Stamina interaction
```

Strength produces Power for scaling calculations.

Base:

```text
1 Strength = 7 Strength Power
```

---

# 25. Dexterity

Dexterity affects:

```text
damage with most melee/body weapons
evasion rate
turn Priority
```

Higher Dexterity contributes to acting earlier in turn order.

Dexterity produces Power.

Base:

```text
1 Dexterity = 7 Dexterity Power
```

---

# 26. Precision

Precision affects:

```text
long-range weapon damage
applicable spell damage
```

Precision produces Power.

Base:

```text
1 Precision = 7 Precision Power
```

---

# 27. Intelligence

Intelligence is the primary magical offensive attribute.

It affects:

```text
magical damage
access to advanced magic nodes
maximum/available spell complexity or weight
```

Intelligence produces Power.

Base:

```text
1 Intelligence = 7 Intelligence Power
```

---

# 28. Power

The offensive attributes:

```text
Strength
Dexterity
Precision
Intelligence
```

generate:

```text
7 Power per base attribute point
```

Power is then consumed by scaling.

Do not directly treat attribute points as damage unless a specific mechanic explicitly requires it.

Conceptually:

```text
Attribute
    ↓
Power
    ↓
Scaling
    ↓
Damage contribution
```

Each sub-use of an attribute may define different effects and non-punitive diminishing returns.

Diminishing-return curves should be data/config driven whenever practical.

---

# 29. Scaling Grades

Scaling converts relevant Attribute Power into damage.

Core scaling multipliers:

```text
D   = 0.25
C   = 0.50
B   = 0.75
A   = 1.00
S   = 1.50
SS  = 2.00
SSS = 2.50
```

These values should have a single authoritative definition.

Prefer making them configurable rather than duplicating constants across combat systems.

---

# 30. Single Scaling

For a single attribute:

```text
Scaling Damage =
Attribute Power × Scaling
```

Then:

```text
Final Base Damage =
Base Damage + Scaling Damage
```

Example:

```text
Strength = 10
Strength Power = 70
Scaling = B = 0.75

Scaling Damage =
70 × 0.75
= 52.5
```

Fractional results round down:

```text
52
```

If the attack has:

```text
Base Damage = 30
```

then:

```text
Final Base Damage =
30 + 52
= 82
```

This is the damage produced by the base/scaling stage before later damage-pipeline stages modify it.

---

# 31. Multiple Scalings

Attacks, weapons, spells, glyphs, or other damage sources may scale from multiple attributes.

Use:

```text
Scaling Damage =
Σ(Attribute Power × Scaling)
```

Then:

```text
Final Base Damage =
Base Damage + Scaling Damage
```

For example:

```text
Strength Power = 70
Strength Scaling = B = 0.75

Dexterity Power = 35
Dexterity Scaling = C = 0.50
```

Then:

```text
Scaling Damage =
(70 × 0.75) + (35 × 0.50)

Scaling Damage =
52.5 + 17.5

Scaling Damage =
70
```

Therefore, for:

```text
Base Damage = 30
```

the result is:

```text
Final Base Damage = 100
```

The engine should represent multiple scalings as data rather than hardcoding combinations such as "Strength + Dexterity".

---

# 32. HP and Tenacity

Characters have two primary defensive bars:

```text
HP
Tenacity
```

HP is increased primarily by Vigor.

Tenacity is increased primarily by Resistance.

While Tenacity remains active, incoming applicable damage is split:

```text
50% → HP
50% → Tenacity
```

Fractional results round down according to the project's rounding rules.

Tenacity also reduces debuff duration.

True Damage ignores Tenacity.

Therefore True Damage must not be split through the normal Tenacity absorption path.

The engine should model Tenacity as its own resource rather than merely treating it as additional HP.

---

# 33. Shields

Shield processing occurs before Tenacity and HP:

```text
Damage Type
 ↓
Shield
 ↓
Tenacity
 ↓
HP
```

Do not merge Shield and Tenacity into the same mechanic.

They occupy different stages of the damage pipeline and may eventually have different interactions with mods, damage types, effects, and triggers.

---

# 34. Triggers

After damage/resources have been resolved:

```text
On Damage Triggers
 ↓
On HP Change Triggers
 ↓
Death Check
```

These trigger families must remain semantically distinct.

`On Damage` means damage occurred.

`On HP Change` means HP changed.

These are not necessarily equivalent because shields, Tenacity, healing, direct HP manipulation, immunity, and future mechanics may distinguish them.

Death checks happen after the relevant triggers according to the defined pipeline.

Avoid immediately killing/removing an entity halfway through damage processing unless an explicit mechanic requires interrupting the pipeline.

---

# 35. Events

Prefer an explicit event system for combat and modding.

Useful event concepts include:

```text
BeforeDamage
AfterDamage
HpChanged
TenacityChanged
ShieldChanged
EffectApplied
EffectRemoved
TurnStarted
TurnEnded
EntityKilled
SpellCast
GlyphTriggered
```

Not every event must exist immediately.

However, new systems should consider whether their behavior belongs in reusable events rather than direct coupling.

Events are especially important for Lua mods.

Avoid designing an event API that can only support base-game mechanics.

---

# 36. Combat Logs

Because combat is textual, combat calculations should be explainable.

The engine should be capable of producing information such as:

```text
Knight attacks Goblin.

Base Damage: 20
Strength Scaling (A): +70
Equipment: +15
Offensive Modifier: ×1.20
Defense: -18
Shield absorbed: 12
Tenacity received: 39
HP received: 39
```

The exact presentation may change.

The important architectural property is that intermediate calculations are not destroyed before the presentation/debugging layer can inspect them.

This also makes mod debugging substantially easier.

---

# 37. Configuration Over Hardcoding

Before adding a gameplay constant directly to Rust, ask:

```text
Could a player reasonably want to configure this?
Could a game mode reasonably change this?
Could a mod reasonably change this?
```

If yes, strongly prefer external configuration/data or a registry/API.

Examples that should generally be configurable:

```text
HP per Vigor
Tenacity per Resistance
MP per Mana
Power per attribute
scaling multipliers
Tenacity damage split
diminishing returns
soft caps
enemy multipliers
glyph weights
loot probabilities
spawn rates
hardcore rules
```

Rust should implement the rules for interpreting these values.

It should not unnecessarily own the values themselves.

---

# 38. Numeric Design

Do not spread raw `f32`/`f64` calculations throughout gameplay code without considering determinism and rounding behavior.

Combat math must have explicit semantics.

Fractional calculations must produce predictable floor behavior where required.

When changing numeric representation, preserve gameplay behavior and deterministic tests.

Do not silently introduce floating-point behavior that can make identical combat calculations produce inconsistent integer results.

---

# 39. Error Handling

Do not use `unwrap()` or `expect()` on normal player-controlled input paths unless failure truly represents an internal invariant violation.

Player-controlled input includes:

```text
config.toml
saves
mods
JSON
Lua
game mode files
commands
```

Malformed external data should normally return/report an error.

Prefer errors containing actionable context.

Bad:

```text
Invalid data.
```

Better:

```text
Failed to load glyph "blood:ritual":
field "weight" must be >= 0, received -4.
```

---

# 40. Compatibility

Save compatibility and mod API compatibility matter.

When changing serialized structures:

* consider old saves;
* use explicit versions where useful;
* provide migrations when reasonable;
* avoid renaming stable IDs casually;
* do not depend on enum ordering or registry load order for persistent identity.

Saves should include a format/schema version.

Mods should be able to declare relevant compatibility information.

---

# 41. Security Model

This is an offline moddable game.

Do not treat the player as an adversary.

The player owns the machine, executable, configuration, mods, and saves.

Do not implement anti-tamper systems.

Do not waste engineering effort preventing players from cheating in their own local game.

However, distinguish save/config editing from Lua execution.

Lua mods are executable code from the player's perspective and should have a deliberately designed API/environment.

Do not unnecessarily expose filesystem, process execution, native libraries, shell commands, or arbitrary operating-system access through Lua.

Powerful game modding does not require unrestricted OS access.

---

# 42. Testing Requirements

Core combat mathematics must be covered by automated tests.

Tests should include at minimum:

```text
attribute → Power
Power → scaling damage
multiple scaling
floor rounding
HP calculation
Tenacity calculation
Mana calculation
Physical damage
Magical damage
True Damage bypassing Tenacity
Shield processing
Tenacity/HP split
effect Potency comparison
effect Counter tie-break
four-source attribute enhancement limit
damage trigger ordering
HP trigger ordering
death check ordering
```

Damage pipeline ordering should have regression tests.

Mod loading should have tests for valid and invalid data.

Lua API behavior should have tests where practical.

Save serialization/deserialization should have round-trip tests.

---

# 43. Implementation Priorities

When implementing new systems, prioritize in this order:

```text
Correctness
↓
Moddability
↓
Configurability
↓
Clarity
↓
Testability
↓
Performance
```

Performance matters, but this is a terminal-based turn-driven RPG.

Do not sacrifice architecture, modding, or correctness for meaningless micro-optimizations.

Optimize measured bottlenecks.

---

# 44. Dependency Policy

Before adding a dependency, verify that it:

* solves a real problem;
* supports offline runtime use;
* does not require a service;
* does not introduce telemetry;
* is compatible with the project's distribution model;
* does not undermine portability.

Prefer established Rust crates for solved infrastructure problems rather than reinventing parsers, ZIP handling, serialization, terminal compatibility, or Lua embedding without reason.

Keep dependencies reasonably minimal.

---

# 45. Codex Implementation Rules

When modifying this repository:

1. Read the relevant existing code before implementing a replacement.
2. Preserve established architecture unless there is a concrete reason to change it.
3. Do not introduce online services.
4. Do not introduce authentication.
5. Do not introduce telemetry.
6. Do not introduce DRM.
7. Do not make player data opaque.
8. Do not hardcode content that should reasonably be moddable/configurable.
9. Prefer registries and data-driven definitions.
10. Keep simulation independent from terminal rendering.
11. Keep Lua behind an explicit modding API.
12. Preserve the defined damage pipeline.
13. Preserve deterministic floor-rounding rules.
14. Add/update tests when changing gameplay mathematics.
15. Keep saves human-editable after decompression.
16. Maintain portable relative filesystem behavior.
17. Never require Internet access at runtime.
18. Never require a separately installed Lua runtime.
19. Treat `/mods` as part of the core architecture.
20. Treat configuration and modding compatibility as API design.

---

# 46. When Requirements Are Ambiguous

Do not invent major gameplay rules merely to complete an implementation.

If a missing decision can be represented generically/configurably, implement the generic architecture without locking the project into an arbitrary rule.

Prefer:

```text
configurable value
registry
trait
event
data definition
Lua hook
explicit TODO
```

over an undocumented assumption.

Small implementation details may be chosen when they do not materially alter gameplay design.

---

# 47. Definition of Done

A feature is not complete merely because it works in the base game.

For gameplay systems, consider whether it is:

```text
functional
tested
serializable where relevant
configurable where relevant
visible to combat/debug logs
compatible with the event system
accessible to mods where relevant
accessible through Lua where relevant
documented enough to be extended
```

Not every feature requires every property, but modding and configuration must be considered during implementation rather than retrofitted later.

---

# 48. Core Architectural Rule

The central architectural principle of this project is:

> Rust provides the engine. Data defines the content. Lua defines extensible behavior. The player owns everything.

Whenever there is a choice between a closed hardcoded implementation and a reasonably maintainable extensible implementation, prefer the extensible implementation.

The game should remain playable, inspectable, modifiable, shareable, and portable without relying on anything outside the player's local installation.

# 49. Logging and Player-Readable Diagnostics

Logging is a first-class user-facing feature.

Logs are NOT exclusively development/debugging artifacts.

The game is intentionally open, moddable, configurable, and player-owned. Therefore, when something fails, the player should be able to inspect the game's logs and reasonably understand:

```text
what failed
where it failed
which file caused it
which mod caused it
which save caused it
which configuration entry caused it
what the game was doing when it failed
```

A player should not need a debugger, Rust toolchain, IDE, or development build to diagnose ordinary failures.

The release build MUST produce useful logs.

---

# 50. Log Directory

Runtime diagnostic information belongs in a visible root directory:

```text
/logs
```

Example installation:

```text
/game
├── game executable
├── config.toml
├── saves/
├── gamemodes/
├── mods/
├── data/
└── logs/
```

The game should create `/logs` automatically if it does not exist.

Logs must use normal, human-readable files.

Prefer:

```text
.log
.txt
```

Do not store ordinary diagnostic logs exclusively in:

```text
binary formats
databases
compressed proprietary formats
OS-specific hidden directories
remote services
```

The player must be able to open the relevant log with an ordinary text editor.

---

# 51. Runtime Logs

Every normal game session should produce runtime logging.

At minimum, the game should maintain a current/latest log:

```text
/logs/latest.log
```

Historical logs may additionally use timestamps:

```text
/logs/2026-09-10_11-42-18.log
/logs/2026-09-10_15-07-52.log
```

A useful startup log might contain:

```text
[11:42:18] [INFO] Game starting
[11:42:18] [INFO] Version: 0.4.2
[11:42:18] [INFO] Platform: windows-x86_64
[11:42:18] [INFO] Loading config.toml
[11:42:18] [INFO] Loading game mode: hardcore
[11:42:18] [INFO] Discovering mods...
[11:42:18] [INFO] Found 14 mods
[11:42:18] [INFO] Loading mod: base
[11:42:18] [INFO] Loading mod: jagger_magic
[11:42:19] [INFO] Loading save: Krieg.zip
```

Logging should cover important initialization boundaries so that a player can determine approximately where startup failed.

---

# 52. Log Levels

Use conventional severity levels:

```text
TRACE
DEBUG
INFO
WARN
ERROR
FATAL
```

Their intended meanings are:

```text
TRACE
Very detailed execution information.

DEBUG
Technical diagnostic information useful for deeper investigation.

INFO
Normal meaningful events such as loading saves, mods, configs, and game systems.

WARN
Something unexpected or potentially problematic occurred, but execution can continue.

ERROR
A specific operation failed, but the game may still be capable of continuing.

FATAL
The game cannot safely continue.
```

Release builds must NOT disable `INFO`, `WARN`, `ERROR`, or `FATAL` diagnostics.

More verbose `DEBUG` and `TRACE` logging may be configurable.

---

# 53. Player-Readable Errors

Errors must be written for humans first.

Do not produce only:

```text
Error 17
Invalid state
Parse failed
unwrap failed
Lua error
JSON error
```

Instead, include context.

Bad:

```text
[ERROR] JSON parse error
```

Good:

```text
[ERROR] Failed to load enemy definition.

Mod: jagger_monsters
File: mods/jagger_monsters/enemies/dragon.json
Enemy: jagger_monsters:ancient_dragon
Field: max_hp
Reason: expected a positive integer, received "a lot"
```

When practical, include the offending value and field.

The goal is that a normal player who understands the game's configuration/modding system can fix the problem without reading Rust source code.

---

# 54. Configuration Diagnostics

Configuration loading must be logged.

Example:

```text
[INFO] Loading global configuration: config.toml
```

If configuration is invalid:

```text
[ERROR] Failed to parse global configuration.

File: config.toml
Section: combat.tenacity
Field: hp_split
Line: 47
Value: 1.7

Reason:
hp_split must be between 0.0 and 1.0.

Expected example:
hp_split = 0.5
```

When the parser provides line and column information, preserve it.

Do not discard useful parser diagnostics merely to replace them with a generic game error.

The same principle applies to `/gamemodes`.

---

# 55. Save Diagnostics

Save loading and saving must be logged.

Example:

```text
[INFO] Loading save: saves/Krieg.zip
[INFO] Reading character.json
[INFO] Reading world.json
[INFO] Reading inventory.json
```

If loading fails, identify the archive and internal file:

```text
[ERROR] Failed to load save.

Save:
saves/Krieg.zip

Internal file:
inventory.json

Item:
base:greatsword

Field:
durability

Reason:
Expected integer >= 0, received -42.
```

If a save references missing modded content, report the missing ID and, where known, the mod that previously provided it.

Example:

```text
[ERROR] Save references an unknown glyph.

Save: saves/Krieg.zip
Character: Krieg
Glyph ID: blood_magic:blood_sun

Possible cause:
The mod "blood_magic" is missing, disabled, incompatible, or failed to load.
```

Do not reduce this to:

```text
Save corrupted.
```

unless corruption has actually been established.

---

# 56. Mod Loading Diagnostics

Mod loading must be especially verbose and useful.

Log discovery and loading:

```text
[INFO] Discovering mods in /mods
[INFO] Found mod: blood_magic
[INFO] Found mod: better_dragons
[INFO] Loading mod: blood_magic
```

If a mod fails, identify it explicitly:

```text
[ERROR] Failed to load mod.

Mod: blood_magic
Version: 1.4.0
Path: mods/blood_magic
Stage: Glyph registration
File: glyphs/blood_sun.json
```

If possible, continue loading unrelated mods after one mod fails.

A single broken content mod should not necessarily prevent the player from reaching the game or diagnostic interface.

Whether continuing is safe depends on the failure.

---

# 57. Lua Diagnostics

Lua errors must preserve Lua-level information.

Do not hide Lua errors behind generic Rust errors.

Example:

```text
[ERROR] Lua execution failed.

Mod: blood_magic
Script: mods/blood_magic/scripts/blood_ritual.lua
Event: OnDamage
Line: 84

Error:
attempt to perform arithmetic on a nil value

Function:
apply_blood_cost()
```

When available, include a Lua stack trace.

Example:

```text
Lua stack trace:

apply_blood_cost()    blood_ritual.lua:84
on_damage()           blood_ritual.lua:121
<event callback>      combat.lua:17
```

The player/mod author should be able to identify the script and line that failed.

Do not expose only the Rust side of the Lua bridge.

---

# 58. Content Registration Diagnostics

Because the game is data-driven and registry-based, registration failures must identify conflicting content.

Example:

```text
[ERROR] Duplicate registry ID.

Registry: GlyphRegistry
ID: blood_magic:blood_sun

First registered by:
mods/blood_magic/glyphs/blood_sun.json

Conflicting registration:
mods/another_mod/glyphs/blood_sun.json
```

Similarly, unresolved references should be explicit:

```text
[ERROR] Unknown effect reference.

Glyph:
jagger_magic:fire_tornado

Referenced effect:
jagger_magic:burning_winds

Source:
mods/jagger_magic/glyphs/fire_tornado.json
```

Never silently replace registry entries unless the modding API explicitly defines an override mechanism.

---

# 59. Crash Reports

Unexpected fatal failures should produce dedicated crash reports.

Store them under:

```text
/logs/crashes/
```

Example:

```text
/logs/crashes/crash-2026-09-10_11-48-32.txt
```

A crash report should contain as much locally available diagnostic information as practical, including:

```text
game version
build/version information
operating system
architecture
timestamp
current game state/stage
loaded save
selected game mode
loaded mods
mod versions
configuration context
error/panic message
Rust panic location when available
backtrace when available
Lua stack trace when relevant
last relevant game operation
```

For example:

```text
========================================
GAME CRASH REPORT
========================================

Game Version: 0.4.2
Platform: Windows x86_64
Time: 2026-09-10 11:48:32

Current Stage:
Loading Mods

Current Mod:
blood_magic 1.4.0

Current File:
mods/blood_magic/scripts/blood_ritual.lua

Loaded Save:
None

Game Mode:
hardcore

----------------------------------------
ERROR
----------------------------------------

Lua callback registration failed.

blood_ritual.lua:84:
attempt to index nil value 'entity'

----------------------------------------
LOADED MODS
----------------------------------------

base 0.4.2
better_dragons 2.1.0
blood_magic 1.4.0

----------------------------------------
BACKTRACE
----------------------------------------

...
```

---

# 60. Crash Report Visibility

When a crash report is successfully written, tell the player exactly where it is.

Instead of only:

```text
The game crashed.
```

prefer terminal output such as:

```text
The game encountered a fatal error.

Crash report written to:

logs/crashes/crash-2026-09-10_11-48-32.txt

Last runtime log:

logs/latest.log
```

If the game cannot create the crash report, print the diagnostic information directly to the terminal as a fallback.

Failure of the logging system must not turn the original error into an invisible failure.

---

# 61. Panic Handling

Unexpected Rust panics should be captured at the application boundary when practical.

Install an appropriate panic hook so that fatal panics can be written to the crash-report system before termination.

Preserve:

```text
panic message
source location
thread
backtrace when available
current diagnostic context
```

Do not use panic handling as a substitute for normal error handling.

Invalid mods, saves, JSON, TOML, Lua scripts, and player input are expected failure conditions and should normally return structured errors rather than panic.

Crash reports are primarily for genuinely unexpected/fatal failures.

---

# 62. Diagnostic Context

The engine should maintain lightweight diagnostic context describing what it is currently doing.

Conceptually:

```text
Phase: Loading Mods
Mod: blood_magic
File: scripts/blood_ritual.lua
Operation: Registering OnDamage callback
```

or:

```text
Phase: Loading Save
Save: Krieg.zip
File: inventory.json
Entity: player
Item: blood_magic:blood_sword
```

This context should automatically enrich errors and crash reports.

Do not require every error site to manually reconstruct the entire execution context.

---

# 63. Combat Logging

The normal runtime log and the player-facing combat log are related but not identical.

Combat calculations should optionally support detailed diagnostic logging.

For example:

```text
[DEBUG] Damage calculation #1842

Source: player
Target: ancient_dragon
Attack: base:greatsword_heavy

Base: 40

Scaling:
Strength Power: 140
Strength Scaling: S (1.50)
Contribution: 210

Final Base Damage: 250

Equipment Modifier: +20
Active Effects: ×1.10
Offensive Modifier: ×1.25
Defensive Modifier: ×0.80

Damage Type: Physical
Shield absorbed: 32
Tenacity damage: 121
HP damage: 121

Final HP: 842/1200
Final Tenacity: 179/600
```

This is useful for players investigating builds, mods, configuration, or unexpected interactions.

The logging architecture should therefore support inspecting game mathematics rather than treating calculations as black boxes.

---

# 64. Mod List in Diagnostics

Crash reports should include the active mod list.

When practical, runtime logs should also record the resolved mod load order.

Example:

```text
[INFO] Mod load order:

01. base 0.4.2
02. core_magic 1.0.0
03. blood_magic 1.4.0
04. better_dragons 2.1.0
05. jagger_balance 0.8.3
```

This is important because mod interactions may depend on load order.

If dependencies or ordering rules modify the final order, log the resolved result rather than only the discovered filesystem order.

---

# 65. Configuration Snapshot

Crash reports should contain enough configuration information to reproduce relevant problems.

However, avoid blindly dumping enormous configuration files into every report.

Prefer either:

```text
relevant configuration values
```

or a clearly identified configuration snapshot/hash/version where useful.

For gameplay calculation failures, relevant values might include:

```text
HP per Vigor
Tenacity split
scaling table
soft caps
diminishing-return preset
enemy damage multiplier
active game mode
```

The purpose is reproducibility.

---

# 66. Privacy

Logging is local.

Logs and crash reports MUST NOT be automatically uploaded anywhere.

There is:

```text
no telemetry
no crash-report server
no automatic bug-report submission
no analytics endpoint
```

Crash reports belong to the player.

The player may inspect, modify, delete, archive, or voluntarily share them.

The game should never require sharing a crash report.

Do not collect unnecessary personal information.

Do not dump unrelated environment variables, usernames, home-directory contents, IP addresses, machine identifiers, or other irrelevant system information into logs.

Diagnostic information should exist because it helps diagnose the game.

---

# 67. Log Retention

Prevent unlimited accidental log growth.

Historical log retention should be configurable.

Possible configuration:

```toml
[logging]
level = "info"
keep_sessions = 20
keep_crash_reports = 50
combat_debug = false
```

The exact schema may change.

Never delete the current session log while the game is running.

Crash reports should preferably have a separate retention policy from ordinary session logs.

Players must also be able to disable automatic cleanup if they want permanent log history.

---

# 68. Logging Must Not Depend on Internet Access

Logging and crash reporting must remain entirely functional offline.

The complete diagnostic pipeline must work when:

```text
the machine has no network adapter
the machine has never connected to the Internet
the game is running from a USB drive
the player copied the installation from another computer
```

No logging feature may require:

```text
remote symbol servers
cloud logging
remote crash processing
online authentication
external dashboards
```

The local files are the authoritative diagnostic output.

---

# 69. Logging Architecture

Prefer structured internal logging even when the final file is human-readable.

Subsystems should attach context such as:

```text
target = "mod_loader"
mod_id = "blood_magic"
file = "blood_ritual.lua"
event = "OnDamage"
```

rather than constructing every message manually.

This allows the same event to produce a readable line such as:

```text
[ERROR] [mod_loader] [blood_magic]
Failed to register OnDamage callback from blood_ritual.lua.
```

The logging system should be centralized.

Do not create unrelated ad-hoc logging implementations for:

```text
combat
mods
Lua
saves
configuration
world generation
terminal rendering
```

They should feed the same diagnostic infrastructure.

---

# 70. Error Chains

Preserve error causes.

If a high-level operation fails because of a lower-level operation, the diagnostic should explain the chain.

Example:

```text
[ERROR] Failed to load save "Krieg.zip"

Caused by:
Failed to deserialize inventory.json

Caused by:
Failed to resolve item "blood_magic:blood_sword"

Caused by:
Registry contains no item with ID "blood_magic:blood_sword"

Possible cause:
The mod providing this item is missing or failed to load.
```

Do not destroy useful low-level errors when adding high-level context.

---

# 71. Startup Failure Diagnostics

Logging must initialize as early as reasonably possible.

This is important because many failures can happen before reaching the main menu:

```text
config parsing
game mode loading
data loading
mod discovery
mod dependency resolution
JSON parsing
Lua initialization
Lua script loading
registry construction
save discovery
terminal initialization
```

The game should still produce useful diagnostics when startup itself fails.

If normal file logging cannot initialize, use stderr/terminal output as the fallback.

---

# 72. User-Facing Diagnostic Principle

The project follows this rule:

> If the game knows why something failed, the player should be able to know why it failed.

Do not intentionally hide technical information from the player in the name of simplifying the interface.

A concise terminal error may be shown during normal play, while `/logs/latest.log` and crash reports contain the complete details.

For example, the terminal may show:

```text
Failed to load mod "blood_magic".

See logs/latest.log for details.
```

while the log contains:

```text
mod
version
file
line
field
error chain
Lua stack trace
registration stage
```

The player-facing error and technical diagnostic complement each other.

---

# 73. Logging Definition of Done

A system involving external/player-controlled data is not complete until its failures are diagnosable.

This applies especially to:

```text
mods
Lua
JSON content
config.toml
game modes
saves
registries
loot tables
glyphs
enemies
items
effects
world generation
```

When implementing such a system, Codex must consider:

```text
What gets logged when it starts?
What gets logged when it succeeds?
What gets logged when it fails?
Can the player identify the responsible file?
Can the player identify the responsible mod/save/config?
Is the original error preserved?
Would the error still be understandable in a release build?
```

If the answer is no, diagnostic support is incomplete.

Logging is part of the feature, not optional developer instrumentation.
