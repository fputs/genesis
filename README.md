# Genesis

Rust engine and editor for a faithful Pathfinder 2e classic CRPG (isometric / Infinity Engine–style).

This repository ships the engine, editor, and authoring tooling. Campaign narrative is out of scope; MVP uses hand-authored fixture data.

## Docs

Index: [docs/README.md](docs/README.md).

- [Constitution](docs/constitution.md)
- [MVP slice 1](docs/mvp-slice-1.md)
- [MVP architecture](docs/mvp-architecture.md)
- [Project context](docs/project-context.md)
- [Research corpus](docs/research/README.md)
- [Rules corpus](docs/rules-corpus/README.md) (classes, archetypes, spells, substrate roadmap)

## Run

Rust 1.99.1 (`rust-toolchain.toml`). See [TESTER.md](TESTER.md) for the walkthrough and system packages.

```
cargo test
cargo run -p game
cargo run -p game -- --headless
cargo run -p editor -- fixtures/rooms/small_room.json
```

The game loads `fixtures/encounters/room_one.json`: explore the room, then a turn-based fight with Move and Strike.

The editor paints stone, grass, and dirt on a room file and saves it. See [TESTER.md](TESTER.md).

## Status

Workspace crates: `rules`, `content`, `game`, and `editor`. Architecture: [docs/mvp-architecture.md](docs/mvp-architecture.md). Baseline encounter actions are on `main`; next rules work (classes, archetypes, spells) is specified in [docs/rules-corpus/README.md](docs/rules-corpus/README.md).
