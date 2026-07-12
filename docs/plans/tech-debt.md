# Tech Debt & Cross-Cutting Issues

A running log of general problems worth addressing that don't belong to a single feature
plan. Each entry states the problem, why it matters, where it shows up, and (when known) a
sketch of a better approach. Add new items at the bottom; mark them resolved (with the commit
/ PR) rather than deleting, so the history stays visible.

Durable "how/why" design guidance still belongs in `../design/`; this file is for *known
rough edges we intend to revisit*.

---

## 1. Resource paths differ between execution and tests (FAC deck, card data)

**Status:** resolved (testing-plan T3) via dependency injection — `GameEnvironment` +
`FacManager::from_cards`/`from_csv` (options 3 and 4 below).

### Resolution

- Resource loading is centralized in **`GameEnvironment::load`** (`spf/src/game/environment.rs`),
  the single disk-loading site, created once in `main`. `Game` no longer knows about paths:
  `create_game(&env, home, away)` resolves teams and `build(home, away, fac_deck)` takes an
  owned deck (option 3 — DI).
- **`FacManager::from_csv`** returns a `Result`, so a missing/mislocated file is a handled
  startup error instead of a panic (option 4). `FacManager::from_cards` gives tests an
  in-memory deck with no path at all, removing the CWD-relative-path guesswork and the
  self-skip guards (the old `create_game_with_fac_path` workaround is gone).
- See [`../design/game-management.md`](../design/game-management.md) for the layering and the
  ownership model.

Original write-up retained below for history.

---

### Problem (original)

Code that loads on-disk resources uses **CWD-relative paths**, and the correct prefix
differs depending on how the code is run:

- Running the server (`cargo run -p spf`) executes with CWD = **workspace root**, so
  `FacManager::new("cards/fac_cards.csv")` resolves correctly.
- `cargo test` runs a crate's test binary with CWD = the **crate directory** (`spf/`), so the
  same `"cards/fac_cards.csv"` is unreachable; the file is one level up at
  `"../cards/fac_cards.csv"`.

Because the path is **hard-coded inside `create_game`** (`spf/src/game.rs`), a test that
builds a `Game` cannot supply the right prefix, and `FacManager::new` panics on `unwrap()`
(`spf/src/game/fac.rs`) when the file isn't found.

### Where it shows up today

- `Game::create_game` hard-codes `"cards/fac_cards.csv"`.
- The `spf_core` round-trip test (`persist.rs`) already works around the same issue by
  reading `"../cards/SPFB1983"` and **self-skipping** when the fixture is absent
  (`../design/testing-strategy.md` §6).
- **WS Stage 2** hit this: to unit-test event emission we had to refactor `create_game` to
  delegate to a private `create_game_with_fac_path(home, away, fac_path)` and point the test
  at `"../cards/fac_cards.csv"` with a self-skip guard. That is a *local* patch, not a
  general fix. (See `ws-events-stage2.md` → Task 6 note.)

### Why it matters

- Every new place that loads a resource re-invents the "which prefix?" guesswork.
- Tests either need bespoke path-injection seams (like `create_game_with_fac_path`) or must
  self-skip, reducing real coverage.
- The panic-on-missing-file (`unwrap()`) turns a path mistake into a hard crash rather than a
  handled error.

### Sketch of a more elegant approach (to design later)

Options, roughly in order of increasing investment — pick one and apply it consistently:

1. **Resolve paths relative to the crate/workspace, not the CWD.** e.g. anchor on
   `env!("CARGO_MANIFEST_DIR")` (compile-time crate dir) or discover the workspace root once
   at startup, and build resource paths from that anchor. Removes the run-vs-test divergence
   entirely.
2. **A single resource-locating helper / config.** Centralize "where do assets live" in one
   module (or a config value / env var like `SPF_DATA_DIR`) that both the server and tests
   consult, instead of scattering string literals.
3. **Dependency-inject the data source.** Have `Game`/`FacManager` take an already-loaded
   deck (or a reader) rather than a path, so production wires the path in one place and tests
   pass an in-memory deck. This also unlocks the deterministic FAC deck seam that
   testing-plan **T3** wants (`FacManager::from_cards(...)`), so the two efforts should be
   designed together.
4. **Stop `unwrap()`-ing file I/O.** Return `Result` from the loaders so a missing/mislocated
   file surfaces as a handled error instead of a panic.

**Recommendation of record:** fold this into testing-plan **T3** (the FAC deck-injection
seam) — option 3 addresses both the path divergence and the determinism need with one design.
Option 1 is the cheapest partial win if a full DI refactor is deferred.

---
