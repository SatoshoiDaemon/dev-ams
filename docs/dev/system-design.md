# System Design Principles

These are initial development requirements and design intentions, not a claim that every system is implemented. System-specific rules live in the [systems directory](../systems/). The canonical rules are defined by sections and pages marked **Canonical**; proposal and historical text never overrides them.

## Purpose

This document defines the core gameplay systems, progression rules, combat structure, auxiliary systems, build progression, Trials, and endgame activities.

The project is intentionally systemic.

Whenever practical, gameplay features should interact through reusable systems rather than isolated scripted exceptions.

Core design goals:

- Player agency.
- Extremely long-term progression.
- Build experimentation.
- Distinct sources of progression.
- Strong modding support.
- Configurable numerical rules.
- Systems that continue interacting with each other into endgame.
- Avoid arbitrary hard locks when natural mechanical limitations can perform the same role.
- Avoid systems becoming mathematically irrelevant at high levels.

## Configuration Requirements

Values in this document that represent numerical gameplay rules should be configurable unless there is a strong reason otherwise.

Important configurable examples include:

- maximum level
- XP formula parameters
- HP per Vigor
- Tenacity per Resistance
- Mana per Mana attribute
- Power per offensive attribute
- Evasion behavior, when a specific mechanic requires a limit
- Damage Reduction behavior, when a specific mechanic requires a limit
- diminishing-return curves, only when required by their owning mechanic
- quick potion slots
- weapon-ready slots
- Fighting Style slots
- pet limit
- mount limit
- Arena restrictions
- Alchemy success curves
- Fishing timing windows
- Abyss scaling

Balance is not a default architectural requirement. Do not add hard caps, soft caps, diminishing returns, progression ceilings, or arbitrary numeric limits solely for balance. Limits are valid when required by the mechanic itself, by data safety, or by an explicit design decision. In particular, the engine does not impose a universal vertical or horizontal endgame progression cap; individual systems may have natural or structural limits.

The base game provides canonical defaults.

Gamemodes and mods may override them through supported configuration systems.

## Modding Requirements

All major systems should expose explicit extension points.

Mods should eventually be able to add or alter:

- races
- racial passives
- elements
- status effects
- attributes where supported
- alchemy ingredients
- alchemy effects
- fishing loot
- fishing rods
- crafting templates
- weapons
- armor
- trinkets
- Fighting Styles
- Fighting Style Nodes
- Trials
- Shrines
- Arena opponents
- Abyss templates
- bosses
- legendary equipment

Complex behavior may be implemented through embedded Lua.

## System IDs

Persistent gameplay content must use stable IDs.

Examples:

- base:human
- base:dragonborn

- base:fire
- base:lightning

base:basic_closed_combat

base:tidal_fishing_rod

base:excalibur

Do not persist content references using only array indexes or load-order positions.

This is required for:

- save editing
- modding
- compatibility
- debugging
- logging

## Logging Requirements

Systems in this document must integrate with the game's player-readable logging architecture.

Relevant events should be diagnosable.

Examples:

- Failed to load Fighting Style Node.
- Failed to generate Abyss enemy.
- Unknown Element ID in save.
- Invalid Alchemy ingredient definition.
- Fishing loot table references missing item.
- Arena opponent references missing equipment.
- Shrine reward references unknown spell.

The player should be able to identify:

- system
- content ID
- file
- mod
- reason

without requiring a debugger.

## System Separation

Keep separate engine concepts separate.

Examples:

Intelligence ≠ Mana

HP ≠ Tenacity

Fighting Style ≠ Weapon

Fighting Style ≠ Active Skill

Potion Quick Access ≠ Inventory

Element Unlocked ≠ Element Awakened

Character Level ≠ Equipment Progression

Critical Damage ≠ Random Critical Chance

Open World Progression ≠ Endgame Progression

Avoid merging concepts simply because they produce superficially similar numerical results.

## System Interaction Rule

Before implementing a new gameplay feature, consider how it interacts with existing systems.

For example, a new weapon should consider:

- Scaling
- Nodes
- Fighting Styles
- Priority
- Weight
- critical behavior
- equipment slots
- crafting
- templates
- loot
- status effects
- mods
- logging
- serialization

A new spell should consider:

- Elements
- Nodes
- Mana
- Intelligence
- Scaling
- catalysts
- spell weight
- Priority
- status effects
- element awakening
- mods
- serialization

Systems should compose.

Avoid isolated mechanics that cannot participate in the rest of the game architecture.

## Current Canonical System Summary

The current character progression structure can be represented approximately as:

```text
Character Creation
├── Race
└── Up to 2 Elements
```

```text
Character
├── Level
├── Attributes
│   ├── Vigor
│   ├── Resistance
│   ├── Strength
│   ├── Dexterity
│   ├── Precision
│   ├── Intelligence
│   └── Mana
│
├── Resources
│   ├── HP
│   ├── Tenacity
│   └── MP
│
├── Equipment
│   ├── Armor
│   ├── Weapons
│   ├── Catalysts
│   ├── Trinkets
│   └── Talismans
│
├── Combat
│   ├── Actions
│   ├── Priority
│   ├── Bonus Actions
│   ├── Status Effects
│   ├── Critical Damage
│   └── Scaling
│
├── Magic
│   ├── Elements
│   ├── Elemental Nodes
│   ├── Spells
│   ├── Catalysts
│   └── Element Awakening
│
├── Martial Progression
│   ├── Fighting Styles
│   ├── Style Nodes
│   └── Unarmed Weapons
│
├── Professions / Auxiliary Systems
│   ├── Mining
│   ├── Crafting
│   ├── Alchemy
│   └── Fishing
│
└── Endgame
    ├── The Arena
    ├── Knight Shrines
    ├── The Kings
    ├── Elemental Trials
    ├── Trial of Cataclysm
    ├── Honor of the King
    └── Abyss
```

## Core Systems Rule

The central rule of system design is:

Progression should unlock more interactions, not merely produce larger numbers.

Levels and attributes make the character stronger.

Equipment changes how the character fights.

Nodes change how systems behave.

Elements change magical possibilities.

Fighting Styles change combat rules.

Alchemy turns experimentation into consumable power.

Crafting turns exploration into equipment.

Trials unlock new layers of progression.

The Abyss then tests the resulting system against increasingly extreme combinations.

The game should preserve this interaction-heavy structure rather than collapsing late-game progression into simple numerical inflation.
