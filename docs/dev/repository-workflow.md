# Repository Workflow

## Responsibilities

`dev-ams` contains engine development and initial design documentation. `ams` is the public distribution and contribution repository. `wiki-ams` documents systems for players and modders. Each is an independent repository using `main` as its initial branch.

The `dev-ams` repository tracks `git@github.com:Axiom-1337-ts/dev-ams.git` as `origin`, with `main` published as its upstream branch. Hosted repository visibility and branch protection are outside this local setup. The separate public and wiki repositories still require their own remotes.

## Current checks

All three repositories include `scripts/check_docs.py` and a `.github/workflows/docs.yml` workflow. The script uses only the Python standard library and checks required root documents, UTF-8 Markdown, merge markers, closed fenced code blocks, and repository-local inline links and heading fragments. It does not check external URLs, reference-style links, or gameplay correctness.

Run locally from the repository root:

```sh
python scripts/check_docs.py
```

GitHub Actions runs this check on Windows and Linux for pushes, pull requests, and manual dispatch. Workflow tokens have read-only repository permissions. Actions are pinned to commits from the official [checkout release](https://github.com/actions/checkout/releases/tag/v7.0.1) and [setup-python release](https://github.com/actions/setup-python/releases/tag/v7.0.0).

Python and the CI environment are contributor tools. They are not dependencies of the installed game. The local documentation check works without Internet access.

## Future Rust build checks

There is currently no `Cargo.toml`, Rust source, or executable. Rust compilation and release publishing are therefore not part of the initial workflow.

When the engine is added, extend CI to run formatting, Clippy, and appropriate tests, and produce Windows and Linux release builds. The Linux distribution target must include `x86_64-unknown-linux-musl` with static linkage. Verify embedded Lua, relative filesystem paths, complete local assets, and launch from a copied installation with no network connection. Preserve `Cargo.lock` for the application.

Build checks and release packaging must demonstrate these properties before a playable release is advertised. The public repository must distribute the license and the complete portable installation, including `config.toml`, `saves`, `gamemodes`, `mods`, `data`, and `logs` as appropriate.

## Documentation maintenance

Keep one Markdown file per independent system, grouping the internal mechanics of that system together. Use English, relative repository-local links, stable content identifiers, and explicit proposal/open-question labels. Update the documentation index when adding or moving a page.

The migration records document the source locations of the initial specifications. Source file names in those records are historical references, not instructions to recreate the monolithic files.
