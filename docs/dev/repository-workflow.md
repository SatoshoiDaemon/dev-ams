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

## Rust build checks

`.github/workflows/engine.yml` uses the declared Rust 1.80 MSRV to run formatting,
`cargo check --all-targets --all-features`, tests for all targets and features,
and Clippy for all targets and features with warnings denied. It also builds
release executables on Windows and Linux. A separate Linux job installs
`musl-tools`, builds `x86_64-unknown-linux-musl`, and rejects an executable
containing a dynamic program interpreter. `Cargo.lock` is tracked for
reproducible application builds.

Rust's module resolver and the compilation checks validate declared module
paths and reject conflicting `foo.rs`/`foo/mod.rs` definitions. No separate
directory allowlist is used: legitimate crate-root Rust files are allowed, and
new module directories are added only when they represent real responsibilities.

Release publishing is still separate from CI validation. `scripts/package.ps1` and `scripts/package.sh` assemble the portable directory with the executable, license, configuration, data, gamemodes, and visible `saves/mods/logs` directories. CI launches the executable from that copied directory; the musl job also verifies that the Linux executable has no dynamic program interpreter.

Build checks and release packaging must demonstrate these properties before a playable release is advertised. The public repository must distribute the license and the complete portable installation, including `config.toml`, `saves`, `gamemodes`, `mods`, `data`, and `logs` as appropriate.

## Documentation maintenance

Keep one Markdown file per independent system, grouping the internal mechanics of that system together. Use English, relative repository-local links, stable content identifiers, and explicit proposal/open-question labels. Update the documentation index when adding or moving a page.

The current system and world documents are the readable specification. Historical source files and removed monolithic drafts are not part of the active documentation set.
