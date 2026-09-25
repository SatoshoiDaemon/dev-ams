# Technical Contracts

This page defines the minimum technical contracts shared by the systems in `docs/systems`. It does not add gameplay rules. A system page marked **Canonical** is authoritative over proposal and historical text.

## Authority

When documents disagree, use this order:

```text
Canonical > Proposal > Historical
```

- **Canonical**: implementation, tests, saves, logs, and mods must follow it.
- **Proposal**: design material that is not an implementation instruction until promoted.
- **Historical**: preserved provenance only; it never overrides a canonical rule.
- **Open contract**: a technical decision still required before implementation.

Every system page should identify its status near the top. Historical sections should say explicitly that they are retained for provenance.

## Stable IDs

Persistent content uses the form:

```text
namespace:local_id
```

IDs are case-sensitive, must contain exactly one namespace separator, and must not depend on registry order or array position. The `base` namespace belongs to the game. A mod owns IDs in its declared namespace. Duplicate IDs are a load error unless an explicit replacement mechanism is added later.

References in saves, content files, logs, and Lua use stable IDs. Renaming a persistent ID requires a migration alias.

## Content data and validation

Content may use JSON or TOML according to the owning system. Each content type defines a small schema containing its required fields, field types, valid ID references, and bounds. A recipe, for example, only needs inputs, output, and a stable ID; it does not implicitly gain quality, budgets, or failure rules.

Unknown or invalid external data must produce a player-readable diagnostic containing:

```text
mod, file, object ID, field, received value, and reason
```

Invalid content is rejected without a panic. One invalid mod should not make unrelated base content uninspectable, but the load result must clearly identify that the mod was not loaded completely.

## Mod structure, manifest, and load order

Mods use a deliberately simple visible directory structure:

```text
/mods/
    example_mod/
        manifest.json
        content/
            items.json
            weapons.json
            spells.json
        scripts/
            main.lua
```

`content/` and `scripts/` are optional independently. A JSON-only mod is valid;
Lua is required only when the mod adds behavior that declarative data cannot
express. JSON defines content and Lua defines behavior.

Every mod has a `manifest.json` with at least:

```json
{
  "id": "author:example",
  "name": "Example Mod",
  "version": "1.0.0",
  "api_version": 1
}
```

The manifest file format is JSON.

Dependencies may be declared by stable ID. Loading is deterministic:

```text
engine primitives
→ base content
→ dependencies in dependency order
→ independent mods by stable ID
→ each mod's files by stable path order
```

Missing dependencies, dependency cycles, invalid manifests, and duplicate IDs are load errors. A mod may replace existing content only through an explicit replacement declaration; ordinary registration cannot silently overwrite another ID.

## Save ownership and migrations

Save files are player-owned data. Manual editing is a supported use case. The
engine validates whether data can be safely interpreted and executed; it does not
validate whether values are legitimate, reachable, balanced, or obtainable through
normal play.

The engine MUST NOT:

- encrypt saves;
- sign saves;
- prevent manual modification;
- reject a save merely because its values were externally edited;
- enforce gameplay legitimacy or anti-cheat rules.

For example, an externally edited `strength: 999999999` is not corruption merely
because it is unusually large. It is accepted if it is structurally representable
and safe for the current ruleset. Structural failures, malformed JSON, unsupported
schema versions, impossible archive contents, integer overflow, or data that cannot
be safely interpreted may still be rejected with an actionable diagnostic.

Every save records at least:

```json
{
  "save_format_version": 2,
  "ruleset_version": "..."
}
```

`save_format_version` describes serialized structure. `ruleset_version` identifies the gameplay defaults used to interpret it. Modded saves also record the required mod IDs and versions when that information is needed to load their content.

Loading runs migrations in version order. A migration may rename fields or IDs, add
defaults, and emit a warning. A save is rejected only when it cannot be made safe to
load; the diagnostic identifies the save, version, field, and reason. Save data
remains human-readable and editable.

Unknown mod-owned data MUST be preserved as opaque JSON. The engine does not
interpret, semantically validate, or discard it, and writes it back when the save is
written again. If a mod is unavailable, the loader preserves that mod's namespace,
emits a warning, and continues unless a required live reference prevents the
affected operation from being safely loaded.

Unknown data and unresolved live references are distinct:

- an opaque mod subtree may remain untouched while its owner is unavailable;
- a reference such as `krieg_weapons:moonlight_greatsword` must be represented as
  an `UnresolvedContentReference` with its ID and owner namespace;
- the owning system decides whether that unresolved reference blocks the specific
  operation, such as equipping or using the item;
- an absent mod does not automatically invalidate the entire save.

When the mod becomes available again, its namespace data is handed back to that mod
and the mod/API may validate its own semantics. Opaque data is therefore preserved
for round trips, not silently accepted as active gameplay state.

## Lua boundary and API version 1

Lua accesses the game only through the versioned mod API:

```text
mod_api_version = 1
```

API version 1 intentionally starts small and grows with implemented systems. It has
four capabilities:

```text
Registration
Query
Mutation
Events
```

The initial conceptual operations are:

```text
api.register_action(...)
api.register_status(...)

api.get_entity(id)
api.get_attribute(entity, attribute)
api.get_resource(entity, resource)
api.has_status(entity, status)

api.damage(...)
api.heal(...)
api.change_resource(...)
api.apply_status(...)
api.remove_status(...)

api.on("OnDamageReceived", callback)
api.on("OnKill", callback)
```

The exact payload schemas are defined alongside the owning implemented system. API
version 1 does not attempt to publish Fishing, Mining, Abyss, Companions, Alchemy,
or every other future system in advance. New system surfaces are added deliberately
in a later API version when those systems become executable.

The API exposes stable IDs, documented value objects, registration functions, and
event callbacks. It does not expose Rust internals, arbitrary filesystem access,
process execution, shell commands, native libraries, network access, or implicit
operating-system access.

A Lua error identifies the mod, script, line, callback/event, and reason. The failing callback is disabled or the affected operation is rejected according to the owning system; the whole game must not crash because of an ordinary mod error. Callback recursion remains subject to the combat/event safety limits.

## Event contract

Events have a stable name and a documented payload. The core combat event names are:

```text
OnAttackReceived
OnEvade
OnParry
OnBlock
OnDamageReceived
OnShieldDamage
OnTenacityDamage
OnHPDamage
OnHPBelow
OnKill
```

`OnAttackReceived` is the public name for an attack that effectively connected; it is not emitted when Evasion or Parry prevents the attack. `OnDamageReceived` requires damage to actually reduce at least one receiving damage layer; damage completely absorbed by Shield does not emit it. Layer events require that their resource actually decreased. `OnHPBelow` has no universal once/while-below behavior; the effect or listener definition specifies how it consumes the threshold event. Events produced by a Sigil are new events and may activate eligible sources subject to the global recursion and event-count limits.

Event processing follows the owning canonical system's order. Listeners are processed deterministically by registered stable ID. Events created during an event are queued after the current event and remain subject to trigger-depth and event-count limits.

## Targeting

Targeting is selection-based, not spatial. Combat does not simulate distance, positioning, line of sight, line of effect, obstacles, or attack ranges unless a specific action explicitly implements such a requirement.

The canonical target specification is:

```text
TargetSpec:
    mode: Self | Single | Multiple(N) | All
    relation: Self | Ally | Enemy | Any
    state: Active | Dead | Any
    filters: ActionRequirement[]
```

`Self` selects only the acting entity. `Single` requires exactly one valid selected target. `Multiple(N)` accepts up to `N` selected targets. `All` determines the set at resolution rather than storing a concrete list at declaration time. Content may name filtered forms such as `AllEnemies`, `AllAllies`, or `Everyone`; these are relation-based selections, not geometric areas.

By default, only active targets are valid. Dead or removed targets are invalid. Resurrection actions explicitly opt into `state: Dead` rather than changing the generic targeting rules. Relations are evaluated from the actor and target at validation time.

Single and Multiple targets are selected during action declaration. All-target selections are evaluated at resolution. The initial TargetSpec validation happens before AP and resource payment. Every action then revalidates its targets at resolution:

```text
target exists
AND target is active or matches the declared state
AND action permits the target relation
AND action-specific requirements are satisfied
```

Invalid selected targets are removed before resolution. A Single action fails when its target is invalid. A Multiple action resolves against the remaining valid targets and fails only when no valid target remains. An All action resolves against the valid set found at resolution.

Target requirements are declarative and action-specific, such as `target_has_status`, `target_is_boss`, or `target_hp_below`. They must not introduce a hidden global spatial system.

## Explain Last

`explain last` exposes the latest completed action packet, whether the action resolved or was rejected. It replaces the previous packet; it is not an unbounded combat history. Save format v2 persists it inside an active `combat.json` so a resumed combat retains the same trace. For delayed effects such as Aftershock, `after 1 action` counts the next action resolved by any combatant.

The packet includes, when applicable:

```text
action declaration and validation
actor, source, target IDs, and declared round
AP/resource payment
Cast and eligible round
Priority contributions and queue position
Accuracy, Evasion, Parry, and Block inputs/results
Scaling sources and floored contributions
offensive and defensive modifiers
Defense and penetration
Shield, Tenacity, and HP before/after
events emitted and queued
trigger order, depth, and rejection reason
ruleset version and RNG seed/stream when randomness was used
```

Absorbed, skipped, rejected, or inapplicable stages are represented explicitly rather than silently omitted. A future aggregate `explain turn` command may summarize multiple actions; it is distinct from `explain last`.

## Closed implementation contracts

The following decisions are canonical and no longer open design questions:

- [Fleeing](../systems/combat.md#fleeing) ends combat immediately, grants no combat
  rewards, and has no chance, cost, cooldown, reaction, or encounter exception.
- Module dependency direction is `terminal → app → systems → engine`; content,
  persistence, and modding enter through interfaces. Terminal never owns gameplay
  rules, persistence does not decide gameplay, content describes data, systems
  execute rules, and Lua receives no arbitrary Rust references.
- JSON defines declarative content. Lua defines behavior that JSON cannot express.
  Mods may contain content without `scripts/`, and API version 1 starts with the
  four capabilities Registration, Query, Mutation, and Events.
- Unknown mod-owned save data is preserved as opaque JSON. Unavailable mods produce
  warnings and unresolved references rather than automatic whole-save rejection.
- Deterministic behavior is tested with the same state, action, RNG seed, ruleset,
  and mods producing the same result and `explain last` trace.

These are implementation contracts, not requests for additional endgame or content
design.

## Extension contracts after the vertical slice

The vertical slice closes the engine contracts described above. The following are
extension points for systems that are not executable yet; they are not missing rules
for the current combat path.

### Action-specific validation for future systems

`TargetSpec` defines who may be selected, but each action still needs a declarative
validation contract for requirements that are not targeting itself. Before payment,
an action definition must be able to state, where applicable:

- required and forbidden actor states, statuses, equipment, weapon, catalyst, or
  Fighting Style;
- required resources and the exact payment timing;
- whether the action is available in exploration, combat, or both;
- whether it is offensive, blockable, interruptible, or allowed as a Bonus Action;
- required element ownership, node, learned stable ID, or spell preparation;
- target filters such as status, boss, HP threshold, or relation.

The implemented contract specifies that declaration validation happens
before AP/resource payment, and resolution validation happens again after earlier
actions may have changed the state. Invalid actions need a typed rejection reason
and must not partially pay AP or resources. The current typed vocabulary covers
active state, tags, statuses, Mana, target HP, and target relations. Equipment,
element ownership, catalysts, and Fighting Styles extend the centralized vocabulary
when those systems become executable.

### Module boundaries and concrete interfaces

The dependency direction is expressed by the current Rust modules. The maintained
boundary is:

```text
terminal ─> app ─> systems ─> engine/state
                    ↑          ↑
             content/data   persistence/modding
                    └──── interfaces ────┘
```

The following ownership must be explicit:

- `engine/state`: entity IDs, world state, resources, clocks, RNG, and generic
  state transitions;
- `systems/combat`: actions, turns, queues, damage pipeline, status processing,
  combat outcomes, and combat results;
- `content/data`: schemas, registries, validation, load order, and definitions;
- `persistence`: save archives, migrations, and ruleset/mod metadata;
- `modding/Lua`: versioned registration and callbacks through controlled handles;
- `terminal`: input and rendering only; it must not decide game rules;
- `app`: startup, mode transitions, command routing, and dependency composition.

Systems depend on engine contracts rather than terminal types. Future modules must
preserve this direction instead of moving gameplay rules into the CLI.

### Lua API payloads and expansion

The minimum Lua surface is implemented as API version 1: load-time action/status
registration, stable namespaced event listeners, deterministic ordering, copied
entity/attribute/resource/status queries, and queued damage, healing, resource, and
status commands. Each mod has an isolated runtime; callback instruction, recursion,
and event limits are enforced, and an ordinary callback failure disables only that
listener for the session with contextual diagnostics.

Future executable systems may add equipment, inventory, world, spell-editor, and
other system-specific queries or commands. Those additions require documented
payload/value-conversion schemas and deliberate API compatibility decisions; they
must not expand API v1 by exposing arbitrary engine references.

The API continues to omit filesystem, process, shell, native-library, network, and
arbitrary Rust access.

### Save archive layout

Save format v2 contains `metadata.json`, `ruleset.json`, `entities.json`, optional
`combat.json`, and `mod_data.json`. Later systems may add versioned entries, but must
preserve the human-readable, editable ownership contract and ordered migrations.

### Tests and balance validation

The vertical-slice suite covers formulas, damage and event ordering, target
revalidation, atomic payment, Cast interruption, reactions, Bonus Action limits,
status ticks, Demon boundaries, invalid mod dependencies, Lua listener isolation,
configuration precedence, save migration/round-trips, Victory, and `Fled`.
Deterministic systems satisfy:

```text
same state
+ same action
+ same RNG seed
+ same ruleset
+ same mods
= same result and explain last trace
```

Future systems must add equivalent regression vectors and explanation assertions as
they become executable. Balance tuning may continue through versioned data/rulesets
without reopening the deterministic engine contract.
