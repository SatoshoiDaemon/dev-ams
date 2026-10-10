# A Magic Sovereign — Development

A Magic Sovereign is an open-source, offline-first RPG designed to run directly in a terminal. Rust provides the engine, data defines the content, and embedded Lua provides extensible behavior. Players own their saves, configuration, and mods.

The repository now contains a keyboard-driven terminal flow for character creation, a declarative prologue, Tidal Town exploration, and the Demon combat demonstration. Campaign and character data are stored as editable JSON in versioned ZIP saves. Four starter spells and race-specific mechanics remain content/design dependencies; the game reports these gaps instead of inventing them. See the [implementation status](docs/dev/vertical-slice-status.md) for the exact boundary.

## Documentation

Start with the [development documentation index](docs/README.md). It organizes gameplay systems, glyphs, world design, and development notes into dedicated Markdown files.

Read [AGENTS.md](AGENTS.md) for the project's architectural requirements and [CONTRIBUTING.md](CONTRIBUTING.md) before making changes.

## Repository roles

| Directory | Purpose |
| --- | --- |
| `dev-ams` | Engine development and initial design specifications; this repository. |
| `ams` | Public distribution, release information, issues, and contributions. |
| `wiki-ams` | Player and modder documentation for game systems and the modding API. |

Each directory is an independent Git repository. This repository tracks `git@github.com:Axiom-1337-ts/dev-ams.git` as `origin`; the public and wiki repositories require their own remotes.

## Runtime requirements

- The game must launch directly in a terminal on Windows and Linux.
- Linux releases must use static musl builds, especially `x86_64-unknown-linux-musl`.
- The installed game must be portable, including embedded Lua and all required assets.
- Runtime operation must work offline, without accounts, telemetry, DRM, a browser, or a local server.
- Saves must be editable JSON inside ZIP archives. Configuration, mods, data, and logs must remain visible in the installation directory.

GitHub Actions verifies native Windows/Linux builds and the static `x86_64-unknown-linux-musl` release target. The installed game still requires the data and configuration files beside the executable.

## Rust checks

The project MSRV is Rust 1.80. From the repository root:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
cargo build --release --locked
```

Base actions, entities, encounters, statuses, races, scenes, and locations are loaded from `data/content/`. Glyph cards are loaded from `data/glyphs.json`; they are content rather than Rust match tables.

## Play the vertical slice

Run `cargo run --release` (or `ams`) to open the TUI. **Jogar** collects a save identity separately from the Unicode character name, then lets the player choose a race and up to two elements before starting the prologue. Press Enter/Space to advance; Esc opens the skip confirmation. Completing or skipping the prologue sets day 7 in Tidal Town. Exploration currently supports local navigation and read-only character, inventory, spell, and equipment screens. Rest has no action because no rest rules exist yet. Race mechanics and the four standard Node-built spells are explicitly marked as unfinished content dependencies. The Demon training encounter remains available through the demonstration CLI flow.

`config.toml` controls the ritual presentation with `[presentation] visual_effects = "full" | "reduced" | "off"`; the default is `reduced`.

The original line-oriented interface remains available with `ams --cli`:

```text
cargo run --release -- --cli
new demo base:standard
combat base:training-encounter
actions
use base:black-flame-orb-demo base:raider-1
end
save
explain last
```

The TUI and CLI route gameplay through the same structured application operations. `explain last` prints the latest accepted or rejected action trace. New campaign saves use format v3; v1/v2 demonstration saves remain readable and load without invented campaign state.

## Repository checks

With Python 3.12 or newer installed for development, run:

```sh
python scripts/check_docs.py
```

The same check runs in GitHub Actions on Windows and Linux for pushes, pull requests, and manual runs. It checks Markdown encoding, merge markers, fenced code blocks, and local inline links. It makes no network requests. Python is only a contributor tool and is not a planned game runtime dependency.

See the [repository workflow](docs/dev/repository-workflow.md) for the current CI scope and future build requirements.

See the [Rust foundation decisions](docs/dev/rust-foundation.md) for the dependency and initialization contract.

## License

This project is licensed under the [MIT License](LICENSE).
