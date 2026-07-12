# AGENTS.md — Statis Pro Football (SPF)

This file provides guidance for agentic coding assistants operating in this repository.

SPF is a Rust workspace implementing an HTTP server for running **Statis Pro Football**
tabletop game simulations. It uses `actix-web` for the HTTP layer and exposes JSON endpoints
consumed by a front-end client.

---

## Start here: the docs index

Detailed guidance now lives under [`docs/`](docs/). **Read [`docs/README.md`](docs/README.md)
first** — it is the index that tells you which document covers what, split into durable
`design/` references and time-ordered `plans/`.

Jump straight to the reference you need:

| I need to… | Read |
|---|---|
| Run build / lint / test / format / server / data-conversion commands | [`docs/design/commands.md`](docs/design/commands.md) |
| Understand the crate & module layout | [`docs/design/workspace-structure.md`](docs/design/workspace-structure.md) |
| Work on game construction, `GameEnvironment`, or the HTTP-to-domain boundary | [`docs/design/game-management.md`](docs/design/game-management.md) |
| Understand how card data is ingested & loaded | [`docs/design/data-pipeline.md`](docs/design/data-pipeline.md) |
| Follow code style, naming, error handling & architectural patterns | [`docs/design/code-style.md`](docs/design/code-style.md) |
| Work on the HTTP/OpenAPI (utoipa) layer or dev environment | [`docs/design/openapi-utoipa.md`](docs/design/openapi-utoipa.md) |
| Write or run tests | [`docs/design/testing-strategy.md`](docs/design/testing-strategy.md) |
| See what work is planned next | the tables in [`docs/README.md`](docs/README.md) → `plans/` |

---

## Quick orientation

- **Workspace crates:** `spf` (server + game logic), `spf_core` (data model, loaders,
  persistence), `spf_cli` (offline card-data → JSON converter), `spf_macros` (derive macros).
  Full module map: [`docs/design/workspace-structure.md`](docs/design/workspace-structure.md).
- **Data flow:** card `.txt` files are converted **offline** by `spf_cli` into
  `data/1983/*.json`, which the server loads at startup. `fac_cards.csv` is still parsed at
  runtime. Details: [`docs/design/data-pipeline.md`](docs/design/data-pipeline.md).
- **Commands** (full list in [`docs/design/commands.md`](docs/design/commands.md)):
  - Format: `cargo fmt` (check: `cargo fmt -- --check`)
  - Lint: `cargo clippy` (the devcontainer runs this on save)
  - Test: `cargo test --workspace`
  - Run server: `cargo run -p spf`
  - Regenerate data: `cargo run -p spf_cli -- convert --cards-dir cards/SPFB1983 --year 1983`

---

## Conventions when editing docs

- Durable "how / why" guidance → `docs/design/`. Time-ordered "what next" → `docs/plans/`.
- Every new doc under `docs/` must be linked from [`docs/README.md`](docs/README.md).
- If you change a command, crate layout, code-style rule, the data pipeline, or the OpenAPI
  wiring, update the corresponding `docs/design/` file (and this table if a whole doc is
  added/removed).
