# Build, Lint, Test, Format & Run Commands

Single reference for the commands used to work on the SPF workspace. Useful both to humans
running the project locally and to agents operating in the repo.

All commands use standard Cargo. There is no Makefile, npm, or custom script layer.

> The devcontainer configures rust-analyzer to run Clippy on every save
> (`"rust-analyzer.check.command": "clippy"`), so Clippy is the live feedback loop during
> development.

---

## Build

| Command | Purpose |
|---|---|
| `cargo build` | Build the whole workspace (debug). |
| `cargo build --release` | Optimized release build. |
| `cargo build -p spf` | Build a single crate (`spf`, `spf_core`, `spf_cli`, or `spf_macros`). |

---

## Run

| Command | Purpose |
|---|---|
| `cargo run` | Start the actix-web server. Workspace `default-members` points at `spf`, so bare `cargo run` from the repo root is equivalent to `cargo run -p spf` (loads `data/1983` at startup; serves on **8080**). |
| `cargo run -p spf` | Same as above, explicit. |
| `cargo run -p spf_cli -- convert --cards-dir cards/SPFB1983 --year 1983` | Regenerate the persistent JSON data from card `.txt` files. See [`data-pipeline.md`](data-pipeline.md). |

> If `data/1983` is missing, the server exits with a clear error — run the `spf_cli` convert
> command above first.

---

## Format

| Command | Purpose |
|---|---|
| `cargo fmt` | Apply `rustfmt` (default settings) across the workspace. Run before committing. |
| `cargo fmt -- --check` | CI-equivalent check; fails if anything is unformatted. |

---

## Lint

| Command | Purpose |
|---|---|
| `cargo clippy` | Run Clippy lints (the devcontainer runs this on save). |

---

## Test

| Command | Purpose |
|---|---|
| `cargo test --workspace` | Run the whole test suite. Must always pass. |
| `cargo test -p spf_core` | Test a single crate. |
| `cargo test -p spf_core stats::tests` | Run a single module's tests. |
| `cargo test -p <crate> -- --list` | List tests without running them. |
| `… -- --nocapture` | Append to any test command to see `println!`/`eprintln!` output. |

For the testing philosophy, layers, and conventions, see
[`testing-strategy.md`](testing-strategy.md); for the roadmap, see
[`../plans/testing-plan.md`](../plans/testing-plan.md).
