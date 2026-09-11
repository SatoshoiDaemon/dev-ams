# Contributing to A Magic Sovereign

This repository holds engine development and the initial design specifications. Read [AGENTS.md](AGENTS.md), the [documentation index](docs/README.md), and the document for the system you intend to change.

## Making changes

1. Create a focused branch from `main`.
2. Keep all documentation, comments, examples, and contribution text in English.
3. Place each independent system in its own Markdown file under `docs`. Related parts of one system can share a file. Update the index and cross-references when moving content.
4. Preserve formulas, examples, identifiers, configuration contracts, and distinctions between confirmed rules and proposals. Record unresolved design conflicts explicitly rather than choosing a gameplay rule without a design decision.
5. Run `python scripts/check_docs.py` with Python 3.12 or newer.
6. Submit a focused pull request describing the problem, resulting behavior, relevant design decisions, and validation performed.

Keep early development specifications here. The separate `ams` repository is the public distribution and issue-reporting entry point; `wiki-ams` holds player and modder documentation.

## Engine contributions

The engine has not been scaffolded yet. When implementation begins, preserve the offline, portable terminal architecture and embedded Lua requirement. Keep simulation independent of presentation, use registries and data-driven content, preserve the damage pipeline and deterministic floor rounding, and provide useful local diagnostics.

Gameplay mathematics require automated tests. Changes to player-controlled data paths require useful validation errors and compatibility considerations. Do not add online services, authentication, telemetry, DRM, or an external Lua runtime.

Future build checks must cover Windows and static Linux musl releases. A documentation-only check does not validate game builds.

## Reporting problems

For a documentation issue, identify the file, section, conflicting wording, and suggested correction. For an implemented game issue, provide the game version, operating system, steps to reproduce, expected and actual behavior, and relevant mod/configuration context. Share logs and saves only when you choose to; review their contents before posting.

Contributions are provided under the repository's [MIT License](LICENSE).
