# Testing Strategy

Living reference for how we test the SPF workspace. This document describes the *approach*
— philosophy, structure, conventions — and is expected to grow and be revised as the testing
framework matures. It deliberately does **not** track which modules or how many tests exist
today; for that, run `cargo test --workspace` (or `cargo test -p <crate> -- --list`), or see
the time-ordered history in `../plans/testing-plan.md`.

The concrete, time-ordered roadmap of testing work lives separately in
`../plans/testing-plan.md`; keep forward-looking "we will do X next" items there, and keep
durable "how we test" guidance here.

---

## 1. Philosophy

- **Safety net first, coverage second.** The codebase has a rich rules engine. The immediate
  goal is a fast, reliable net that catches regressions in the pure logic, then broadening
  outward to integration and end-to-end.
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

When deciding what to cover next, favor (in rough order):

1. **Pure functions over I/O-bound code.** Parsers, validators, and state-transition
   functions (e.g. anything returning `Result<_, String>` from plain data) are cheap to test
   and the most likely to harbor subtle rules bugs.
2. **Hand-written parsing/validation over generated or trivial code.** Hardcoded fixup
   tables, `from_str` implementations with alias maps, and range/format parsing are exactly
   the places off-by-one and typo bugs hide.
3. **State machines over one-shot calculations.** Functions that take a `State` and produce a
   new `State` (down/score/possession/clock transitions) benefit most from table-driven case
   coverage because the branch count is high and easy to under-test.
4. **Larger, denser modules over small ones**, all else equal — more logic per file means
   more latent bugs per test written.
5. **Defer** anything that needs real fixture data, a full HTTP harness, or the FAC
   nondeterminism seam (see §5) until those seams exist; track such work in
   `../plans/testing-plan.md` rather than testing around the gap ad hoc.

Concrete "next up" targets are a planning concern, not a strategy concern — see the stage
tables in `../plans/testing-plan.md`.

---

## 5. Determinism: the FAC deck seam

The **only** source of nondeterminism in the engine is the FAC deck shuffle in
`spf/src/game/fac.rs`.

The seam now exists. `FacManager` separates *how the deck is sourced* from *whether it
shuffles*:

- **`FacManager::from_cards(Vec<FacCard>)`** — builds a deck from an explicit, ordered card
  list and draws it in that order **without shuffling** (and re-draws in the same order across
  refills). This is the injection point for deterministic tests.
- **`FacManager::from_csv(path)`** — the production path; loads from CSV and shuffles on
  refill (surfacing I/O errors as a `Result` instead of panicking).

Downstream, `Game::build(home, away, fac_deck)` is a pure constructor that accepts an owned
`FacManager`, so a test can build a fully deterministic game:

```rust
let deck = FacManager::from_cards(vec![/* known, ordered cards */]);
let game = Game::build(home_roster, away_roster, deck); // reproducible draws
```

See [`game-management.md`](game-management.md) for how construction and the environment are
wired. Tests that don't inject a deck must still avoid asserting on card-draw-dependent
output.

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

## 7. Running tests

- Whole workspace: `cargo test --workspace`
- One crate: `cargo test -p spf_core`
- One module: `cargo test -p spf_core stats::tests`
- See `println!` output: append `-- --nocapture`
- List without running: `cargo test -p <crate> -- --list`

Formatting and lints are part of keeping the suite healthy: `cargo fmt -- --check` and
`cargo clippy` (the devcontainer runs clippy on save).
