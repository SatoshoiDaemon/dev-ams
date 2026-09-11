# A Magic Sovereign — Development

A Magic Sovereign is an open-source, offline-first RPG designed to run directly in a terminal. Rust provides the engine, data defines the content, and embedded Lua provides extensible behavior. Players own their saves, configuration, and mods.

This repository currently contains the initial design documentation and repository tooling. There is no Rust application, playable build, or release artifact yet. Design proposals and unresolved rules are identified in the documentation.

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

These are implementation requirements, not claims about an existing build.

## Repository checks

With Python 3.12 or newer installed for development, run:

```sh
python scripts/check_docs.py
```

The same check runs in GitHub Actions on Windows and Linux for pushes, pull requests, and manual runs. It checks Markdown encoding, merge markers, fenced code blocks, and local inline links. It makes no network requests. Python is only a contributor tool and is not a planned game runtime dependency.

See the [repository workflow](docs/dev/repository-workflow.md) for the current CI scope and future build requirements.

## License

This project is licensed under the [MIT License](LICENSE).
