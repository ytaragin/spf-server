# Code Style & Architectural Patterns

Durable coding conventions for the SPF workspace, plus the key architectural patterns you
should follow when extending the code. This is the "how we write code here" reference.

For testing conventions specifically, see [`testing-strategy.md`](testing-strategy.md).

---

## Formatting

- Formatter: `rustfmt` with default settings (no `rustfmt.toml` override).
- Run `cargo fmt` before committing. CI equivalent: `cargo fmt -- --check`.
- 4-space indentation; no tabs.
- Trailing commas in multi-line struct/enum/function-call expressions.

---

## Lints & warnings

- Lint policy is centralized in the root `Cargo.toml` under `[workspace.lints.rust]`; each member
  crate opts in with `[lints] workspace = true`. Change the policy there, not per-crate.
- The policy sets `unused = "warn"` (kept at `warn`, not `deny`, so local builds are not blocked).
  CI may escalate to a hard failure with `cargo clippy --workspace -- -D warnings`.
- Keep the build **warning-free**. If code must stay in-tree while unused, annotate it narrowly with
  an item- or field-level `#[allow(dead_code)]` plus a one-line reason:
  - `// unused: <why> ; kept pending removal.` for leftover/duplicate code,
  - `// TODO(<feature>): <what unblocks it>` for parked-feature scaffolding,
  - `// FIXME: <defect>` for a known bug that is being deferred rather than fixed.
- Prefer fixing the warning (remove the code, wire it up, or rename an unused binding to `_name`)
  over blanket crate-level `#![allow(...)]`.

---

## Naming conventions

| Element | Convention |
|---|---|
| Files / modules | `snake_case` |
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

---

## Imports

- Use `use crate::...` for absolute intra-crate paths.
- Use `use super::...` for parent-module relative paths.
- External crate imports use the crate name directly (`use actix_web::...`, `use serde::...`).
- Proc-macro derives from `spf_macros` are brought in with `use spf_macros::...` where used (an
  `extern crate spf_macros;` declaration is *not* needed under edition 2021).
- Group imports: std first, then external crates, then internal (`crate::`/`super::`).
- No path aliases are configured; all paths are explicit.

---

## Types

- Prefer concrete types over `Box<dyn Trait>` unless dynamic dispatch is required.
- Use type aliases (`type Yard = i32;`) for domain-specific primitive types to improve readability.
- Avoid `unwrap()` and `expect()` in production logic paths; reserve them for startup initialization
  and data loading where a failure is truly unrecoverable.
- No external error-handling crates (`thiserror`, `anyhow`) are used; keep this consistent unless
  adding one is explicitly discussed.

---

## Error handling

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

---

## Comments

- Use `//` for single-line comments; `//!` for module/crate-level doc comments; `///` for item
  doc comments.
- Avoid leaving large blocks of commented-out code in committed changes; use git history instead.

---

## Key architectural patterns

- **`PlayImpl` trait** — the central abstraction for executing a play. All play types implement this
  trait. Adding a new play type means implementing `PlayImpl`.
- **`lazy_static!` constants** in `engine/defs.rs` — game constants (yardage tables, dice lookup
  tables) are initialized once at startup. Add new tables there.
- **Custom derive macros** in `spf_macros` — `#[derive(ImplBasePlayer)]`, `#[derive(IsBlocker)]`,
  etc. generate boilerplate for player stat structs. Prefer these over hand-written impl blocks.
- **Serialization** — `serde::Serialize`/`Deserialize` are derived on most structs. Keep all
  public-facing types serializable. Use `#[serde(rename = "...")]` if the JSON key must differ from
  the Rust field name. (The persistence layer depends on this — see
  [`data-pipeline.md`](data-pipeline.md).)
