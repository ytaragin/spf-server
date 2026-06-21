# AGENTS.md — Statis Pro Football (SPF)

This file provides guidance for agentic coding assistants operating in this repository.

---

## Project Overview

SPF is a Rust workspace implementing an HTTP server for running **Statis Pro Football** tabletop game
simulations. It uses `actix-web` for the HTTP layer and exposes JSON endpoints consumed by a front-end
client.

### Workspace Structure

```
spf/                  # Workspace root
├── spf/              # Main server crate (actix-web server, game logic)
│   └── src/
│       ├── main.rs
│       ├── webendpoint.rs       # HTTP handlers, route scopes, OpenAPI (utoipa) wiring
│       ├── game.rs              # Top-level Game struct + GameState
│       └── game/
│           ├── engine.rs        # Core play-execution traits and types
│           ├── engine/
│           │   ├── defs.rs          # Constants and lookup tables (lazy_static)
│           │   ├── runplay.rs       # Run play logic
│           │   ├── passplay.rs      # Pass play logic
│           │   ├── kickplay.rs      # Kickoff play logic
│           │   ├── playutils.rs     # Shared play utilities and logging macros
│           │   └── resulthandler.rs # Post-play state (down, score, possession)
│           ├── standard_play.rs     # StandardPlay struct + call types
│           ├── kickoff_play.rs      # KickoffPlay struct + PlayImpl
│           ├── players.rs           # Player stat structs, Roster, TeamList, BasePlayer trait
│           ├── lineup.rs            # OffensiveBox/DefensiveBox enums + lineup structs
│           ├── loader.rs            # File parsers for player stat text files
│           ├── fac.rs               # FAC card deck: parsing, shuffling, data types
│           └── stats.rs             # Generic stat types: Range, TwelveStats, RangedStats
└── spf_macros/       # Procedural macro crate
    └── src/lib.rs    # Custom derive macros: ImplBasePlayer, IsBlocker, IsReceiver, etc.
```

---

## Build, Lint, Test, and Format Commands

All commands use standard Cargo. There is no Makefile, npm, or custom script layer.

> The devcontainer configures rust-analyzer to run Clippy on every save
> (`"rust-analyzer.check.command": "clippy"`), so Clippy is the live feedback loop during development.

---

## Code Style Guidelines

### Formatting

- Formatter: `rustfmt` with default settings (no `rustfmt.toml` override).
- Run `cargo fmt` before committing. CI equivalent: `cargo fmt -- --check`.
- 4-space indentation; no tabs.
- Trailing commas in multi-line struct/enum/function-call expressions.

### Naming Conventions

| Element | Convention |
|---|---|
| Files / modules |
| Structs | `PascalCase` |
| Enums | `PascalCase` |
| Enum variants | `PascalCase` |
| Traits | `PascalCase` |
| Functions / methods | `snake_case` |
| Variables / locals | `snake_case` |
| Constants / statics | `SCREAMING_SNAKE_CASE` |
| Type aliases | `PascalCase` |

Do **not** mix `PascalCase` and underscores in type names (e.g. avoid `Serializable_Roster` — this
violates Rust conventions and triggers Clippy warnings).

### Imports

- Use `use crate::...` for absolute intra-crate paths.
- Use `use super::...` for parent-module relative paths.
- External crate imports use the crate name directly (`use actix_web::...`, `use serde::...`).
- `extern crate spf_macros;` is declared in `main.rs` for the proc-macro crate; do not add this
  declaration elsewhere.
- Group imports: std first, then external crates, then internal (`crate::`/`super::`).
- No path aliases are configured; all paths are explicit.

### Types

- Prefer concrete types over `Box<dyn Trait>` unless dynamic dispatch is required.
- Use type aliases (`type Yard = i32;`) for domain-specific primitive types to improve readability.
- Avoid `unwrap()` and `expect()` in production logic paths; reserve them for startup initialization
  and data loading where a failure is truly unrecoverable.
- No external error-handling crates (`thiserror`, `anyhow`) are used; keep this consistent unless
  adding one is explicitly discussed.

### Error Handling

- The dominant pattern is `Result<T, String>` — return `Err("descriptive message".to_string())`.
- Convert `Option` to `Result` with `.ok_or("message")` or `.ok_or_else(|| ...)`.
- Use the `?` operator for error propagation throughout method chains.
- At HTTP handler boundaries, match on `Result` to return appropriate `HttpResponse`:
  ```rust
  match game.some_operation() {
      Ok(_)    => HttpResponse::Ok().body("Success"),
      Err(msg) => HttpResponse::BadRequest().body(msg),
  }
  ```
- In procedural macros (`spf_macros`), `panic!` is acceptable for invalid derive targets since that
  is a compile-time error.

### Comments

- Use `//` for single-line comments; `//!` for module/crate-level doc comments; `///` for item
  doc comments.
- Avoid leaving large blocks of commented-out code in committed changes; use git history instead.

---

## Testing

The codebase currently has no tests. When adding tests:

- **Unit tests:** Add an inline `#[cfg(test)] mod tests { ... }` block at the bottom of the file
  under test (standard Rust idiom).
- **Integration tests:** Place files in `spf/tests/` (Cargo integration test directory).
- Name test functions descriptively using `snake_case`: `test_run_play_returns_correct_yardage`.
- Use `-- --nocapture` to see `println!` output when debugging a failing test.
- No test-specific external dependencies are currently declared; add `[dev-dependencies]` to
  `spf/Cargo.toml` if needed.

---

## Key Architectural Patterns

- **`PlayImpl` trait** — the central abstraction for executing a play. All play types implement this
  trait. Adding a new play type means implementing `PlayImpl`.
- **`lazy_static!` constants** in `engine/defs.rs` — game constants (yardage tables, dice lookup
  tables) are initialized once at startup. Add new tables there.
- **Custom derive macros** in `spf_macros` — `#[derive(ImplBasePlayer)]`, `#[derive(IsBlocker)]`,
  etc. generate boilerplate for player stat structs. Prefer these over hand-written impl blocks.
- **Serialization** — `serde::Serialize`/`Deserialize` are derived on most structs. Keep all
  public-facing types serializable. Use `#[serde(rename = "...")]` if the JSON key must differ from
  the Rust field name.

---

## Development Environment

- **Recommended:** Use the provided DevContainer
- **Ports:** The server runs on **8080**; a companion front-end (if present) runs on **3000**.
  Both are forwarded by the devcontainer.
- **OpenAPI / Swagger (utoipa)** — the API spec is *generated from code*, not hand-written. There is
  no `swagger.yaml`; it was retired in favour of `utoipa` + `utoipa-actix-web` +
  `utoipa-swagger-ui` (see `spf/Cargo.toml`). Key conventions:
  - **Handlers** in `webendpoint.rs` use actix attribute macros (`#[get("...")]` / `#[post("...")]`)
    plus a `#[utoipa::path(...)]` annotation, and are registered with `.service()`. The
    `actix_extras` feature lets utoipa infer the URL and path params from the actix macro, so the
    path string is written once. Routes are grouped into resource scopes (`/game`, `/offense`,
    `/defense`) via `utoipa_actix_web::scope::scope(...)`, which auto-prefix the generated paths.
  - **Schemas** — types that appear in request bodies or responses derive `utoipa::ToSchema`
    (alongside serde). `ToSchema` only *describes* a type's shape for the spec; it does not replace
    serde, which still does the runtime (de)serialization. utoipa honors serde attributes
    (`rename`, `default`, `untagged`, …) when emitting schemas. Query-param structs derive
    `IntoParams`. Fidelity is tiered: deep read-only graphs (player stat structs, full play
    internals) are left opaque via `#[schema(value_type = Object)]` rather than fully annotated.
  - **Manual schemas** — `OffenseCall` and `DefenseCall` (in `engine.rs`) use a custom
    `impl_deserialize!` macro instead of a real `#[serde(untagged)]`, so they have hand-written
    `ToSchema` impls that emit a `oneOf` of the inner call structs. Adding a new variant means
    updating that `oneOf`.
  - **`ApiDoc`** (in `webendpoint.rs`) carries base info (title/version/servers) and lists the two
    manually-implemented call enums plus their inner variant schemas under
    `components(schemas(...))`. Everything else auto-collects from `.service()` calls.
  - **Wiring gotcha** — in `runserver()`, `.openapi(ApiDoc::openapi())` *replaces* the wrapped spec,
    so it must be called **before** the `.service()` calls (it prepends base info; services then add
    paths/schemas on top). Middleware (CORS) is applied via `.map(|a| a.wrap(cors))`. Swagger UI is
    mounted at `/swagger-ui/` and the raw spec at `/api-docs/openapi.json` after `split_for_parts()`.
- **API exploration:** Browse the live API at `http://127.0.0.1:8080/swagger-ui/` (raw spec at
  `/api-docs/openapi.json`). A Postman collection (`spf.postman_collection.json`) at the workspace
  root also documents all HTTP endpoints.
- **Game data:** Player card data lives in `cards/SPFB1983/` (text files) and `cards/fac_cards.csv`.
  These are parsed at startup by `game/loader.rs` and `game/fac.rs`.
