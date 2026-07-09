# Testing Strategy

Living reference for how we test the SPF workspace. This document describes the *approach*
— philosophy, structure, conventions, and current inventory — and is expected to grow and
be revised as the testing framework matures.

The concrete, time-ordered roadmap of testing work lives separately in
`../plans/testing-plan.md`; keep forward-looking "we will do X next" items there, and keep
durable "how we test" guidance here.

---

## 1. Philosophy

- **Safety net first, coverage second.** The codebase is ~8,000 lines with a rich rules
  engine. The immediate goal is a fast, reliable net that catches regressions in the pure
  logic, then broadening outward to integration and end-to-end.
- **Highest ROI first.** Prefer pure, dependency-free functions (parsers, validators, state
  transitions) before I/O- or network-bound paths. They are the cheapest to test and the
  most likely to harbor subtle rules bugs.
- **Determinism is a feature.** Tests must not depend on wall-clock time, network, or
  unseeded randomness. Where the production code is nondeterministic (see §5), we introduce
  a testing seam rather than asserting on random output.
- **Every change stays green.** `cargo test --workspace` must pass at all times. New tests
  land with the code they cover.

---

## 2. Test layers

| Layer | Location | Purpose | Notes |
|---|---|---|---|
| **Unit** | inline `#[cfg(test)] mod tests { … }` at the bottom of the module under test | Exercise a single pure function / small unit | Standard Rust idiom; no `tests/` file needed |
| **Integration** | `spf/tests/` (and `spf_core/tests/` if needed) | Exercise a crate through its public surface, incl. the actix `App` via `test::init_service` | Cargo integration-test directory; each file is its own crate |
| **Round-trip / fixture** | inline or integration | Parse → persist → reload style checks against real card data | Must self-skip when fixture data is absent (see §6) |

There are currently no benchmarks or property-based tests; add sections here if/when they
are introduced.

---

## 3. Conventions (aligned with `AGENTS.md`)

- **Unit tests:** inline `#[cfg(test)] mod tests { use super::*; … }` at the bottom of the
  module under test.
- **Integration tests:** files under `spf/tests/`.
- **Naming:** descriptive `snake_case`, prefixed `test_`, e.g.
  `test_run_play_returns_correct_yardage`.
- **Structure:** prefer table-driven cases for parsers/validators (a slice of
  `(input, expected)` iterated with clear assertion messages).
- **Assertion messages:** include context in multi-case loops so a failure identifies the
  offending case.
- **Debugging output:** `println!`/`eprintln!` is captured by default; run with
  `-- --nocapture` to see it.
- **Dev dependencies:** none are required today. Add `[dev-dependencies]` to the specific
  crate's `Cargo.toml` only when a test first needs one (e.g. stream helpers for WS
  integration tests); do not add them speculatively. This is consistent with the project's
  "no speculative deps" stance.
- **No new error-handling crates** in tests either — the codebase avoids `anyhow`/`thiserror`;
  tests use `expect(...)` / `unwrap()` freely since a panic *is* a test failure.

---

## 4. What we prioritize testing

The most logic-dense modules are the highest-value targets (pure functions,
`Result<_, String>` returns, little or no I/O):

| File | Lines | Test-worthy logic |
|---|---|---|
| `spf_core/src/lineup.rs` | 887 | `is_legal_lineup`, `validate_count`, `count_array_spots`, `OffensiveBox`/`DefensiveBox` `from_str` |
| `spf_core/src/players.rs` | 834 | `TeamID::create_from_str` (hardcoded name-fixup table), stat lookups |
| `spf/src/game/engine/passplay.rs` | 548 | pass resolution |
| `spf_core/src/loader.rs` | 508 | text parsers for player stat files |
| `spf/src/game/engine/runplay.rs` | 435 | run resolution |
| `spf/src/game/engine/defs.rs` | 412 | lookup tables / constants |
| `spf/src/game/standard_play.rs` | 369 | play validation |
| `spf/src/game.rs` | 363 | `GameState` transitions (`get_next_move_types`, `set_next_play_type` legality) |
| `spf_core/src/stats.rs` | 298 | `Range` / `RangedStats` parsing (e.g. `"12-18"`) |
| `spf/src/game/engine/resulthandler.rs` | 130 | `calculate_play_result`: `(GameState, PlayResult)` → new `GameState` (down/score/possession) |

Update this table as modules are covered or as the code evolves.

---

## 5. Determinism: the FAC deck seam

The **only** source of nondeterminism in the engine is the FAC deck shuffle:

- `spf/src/game/fac.rs:220` — `self.deck.shuffle(&mut thread_rng());`

Because of this, `Game::run_current_play` cannot currently be asserted deterministically.
The strategy for testing anything downstream of card draws is to **inject a known deck**
rather than to assert on shuffled output:

- **Preferred seam:** a constructor such as `FacManager::from_cards(Vec<...>)` /
  `with_deck(...)` that builds a deck from an explicit, ordered card list, bypassing the
  shuffle. Tests then get reproducible `PlayResult`s.
- **Alternative (heavier, not currently pursued):** inject a seeded RNG
  (`StdRng::seed_from_u64`) in place of `thread_rng()`.

This seam does not exist yet; see `../plans/testing-plan.md` for when it lands. Until then, tests
must avoid asserting on card-draw-dependent output.

---

## 6. Fixture data

Some tests depend on the raw card data under `cards/SPFB1983/`. This data may be absent in
a lean checkout, so **fixture-dependent tests must self-skip when the data is missing**
rather than fail. The existing round-trip test demonstrates the pattern:

```rust
let cards_dir = "../cards/SPFB1983";
if !Path::new(cards_dir).join("83QB.txt").exists() {
    eprintln!("skipping round-trip test: {} not present", cards_dir);
    return;
}
```

For tests that need a game/roster but not the full league, prefer a small in-code
`TeamList`/`Roster` builder over loading real data (to be introduced as integration testing
expands).

---

## 7. Current inventory

> Snapshot; refresh when tests are added or removed. Command:
> `cargo test --workspace` (see per-crate breakdown with
> `cargo test -p <crate> -- --list`).

| Crate | Location | Tests |
|---|---|---|
| `spf_core` | `src/persist.rs` | `test_sanitize_file_stem_replaces_unsafe_chars`, `test_convert_and_reload_round_trip` (self-skips without card data) |
| `spf_core` | `src/stats.rs` | 7 `Range` unit tests (parsing, `in_range` inclusivity, `get_tag_and_range`) |
| `spf` | — | none yet |
| `spf_cli` | — | none yet |
| `spf_macros` | — | none yet |

**Total: 9 tests**, all in `spf_core`. No integration (`tests/`) directories, no
`[dev-dependencies]`, no CI gate yet.

---

## 8. Running tests

- Whole workspace: `cargo test --workspace`
- One crate: `cargo test -p spf_core`
- One module: `cargo test -p spf_core stats::tests`
- See `println!` output: append `-- --nocapture`
- List without running: `cargo test -p <crate> -- --list`

Formatting and lints are part of keeping the suite healthy: `cargo fmt -- --check` and
`cargo clippy` (the devcontainer runs clippy on save).
