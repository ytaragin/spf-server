# Workspace Structure

Durable reference for how the SPF Cargo workspace is laid out and what each crate/module is
responsible for. Read this to orient yourself before navigating the code.

SPF is a Rust workspace implementing an HTTP server for running **Statis Pro Football**
tabletop game simulations. It uses `actix-web` for the HTTP layer and exposes JSON endpoints
consumed by a front-end client.

---

## Crate & module map

```
spf/                  # Workspace root
├── spf/              # Main server crate (actix-web server, game logic)
│   └── src/
│       ├── main.rs                 # Loads persistent data (data/1983) then starts server
│       ├── webendpoint.rs          # HTTP handlers, route scopes, OpenAPI (utoipa) wiring
│       ├── game.rs                 # Top-level Game struct + GameState; re-exports spf_core model
│       └── game/
│           ├── engine.rs           # Core play-execution traits and types
│           ├── engine/
│           │   ├── defs.rs          # Constants and lookup tables (lazy_static)
│           │   ├── runplay.rs       # Run play logic
│           │   ├── passplay.rs      # Pass play logic
│           │   ├── kickplay.rs      # Kickoff play logic
│           │   ├── playutils.rs     # Shared play utilities and logging macros
│           │   └── resulthandler.rs # Post-play state (down, score, possession)
│           ├── standard_play.rs     # StandardPlay struct + call types (re-exports PassResult etc.)
│           ├── kickoff_play.rs      # KickoffPlay struct + PlayImpl
│           └── fac.rs               # FAC card deck: parsing, shuffling, data types
├── spf_core/         # Shared library crate: data model, loaders, persistence
│   └── src/
│       ├── lib.rs
│       ├── players.rs           # Player stat structs, Roster, TeamList, BasePlayer, Player enum
│       ├── lineup.rs            # OffensiveBox/DefensiveBox enums + lineup structs
│       ├── loader.rs            # File parsers for player stat text files
│       ├── stats.rs             # Generic stat types: Range, TwelveStats, RangedStats
│       ├── shiftable.rs         # Shiftable trait + PassResult/PassRushResult enums
│       └── persist.rs           # Persistent JSON format: write_league / load_league / manifest
├── spf_cli/          # Standalone CLI: converts card .txt files into persistent JSON
│   └── src/main.rs   # `spf-cli convert --cards-dir <dir> --year <yy> --out <dir>`
└── spf_macros/       # Procedural macro crate
    └── src/lib.rs    # Custom derive macros: ImplBasePlayer, IsBlocker, IsReceiver, etc.
```

---

## Related docs

- **Data flow between these crates** (offline ingestion → runtime load):
  [`data-pipeline.md`](data-pipeline.md).
- **Coding conventions & architectural patterns** used throughout:
  [`code-style.md`](code-style.md).
- **HTTP/OpenAPI layer** (`webendpoint.rs`, utoipa wiring):
  [`openapi-utoipa.md`](openapi-utoipa.md).
