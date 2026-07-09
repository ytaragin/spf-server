# Testing Improvement Plan

The time-ordered roadmap for building out the SPF test suite. This is the plan we
*execute*; the durable "how we test" guidance lives in `../design/testing-strategy.md`.

Update the **Status** column and the progress notes as stages land.

---

## Starting point (baseline)

Before this plan began, the workspace had **2 tests**, both in `spf_core/src/persist.rs`:

- `test_sanitize_file_stem_replaces_unsafe_chars` — pure unit test.
- `test_convert_and_reload_round_trip` — parse cards → write JSON → reload; self-skips when
  card data is absent.

There were no `tests/` integration directories, no `[dev-dependencies]`, and no CI gate.

> Correction note: an earlier draft of this plan stated the workspace had "zero tests." That
> was inaccurate — it was a tooling artifact during analysis (the search binary was
> unavailable and failed silently). The two `persist.rs` tests above already existed and
> established the inline `#[cfg(test)] mod tests` + self-skipping-fixture patterns we build
> on.

---

## Stage overview

| Stage | Theme | Status | Outcome |
|---|---|---|---|
| T1 | Harness bootstrap | ✅ Done | `cargo test` runs real assertions in a leaf module; loop proven. |
| T2 | Pure-logic unit tests | 🟡 In progress | `spf_core` targets done (19 tests across `lineup`/`players`/`stats`; +1 alias bugfix). Remainder: `resulthandler.rs` (spf crate) and full `is_legal_lineup` (needs builders). |
| T3 | Deterministic FAC seam | ⬜ Not started | `FacManager` buildable from an explicit deck; play execution reproducible. |
| T4 | HTTP / integration tests | ⬜ Not started | `spf/tests/` exercising the real actix `App` via `test::init_service`. |
| T5 | CI gate | ⬜ Not started | `.github/workflows` running fmt-check + clippy + test on every push. |
| T6 | Warning cleanup | ⬜ Not started | Drive build warnings to zero so real issues aren't lost in noise. |

Sequencing favors fastest safety-net-per-effort. Each stage is independently landable and
leaves `cargo test --workspace` green.

---

## Stage T1 — Harness bootstrap ✅

**Goal:** move from a bare baseline to "green with meaningful, purpose-written coverage" and
confirm the `cargo test` loop works, with no new dependencies.

**What landed:**
- Added `#[cfg(test)] mod tests` to `spf_core/src/stats.rs` with 7 assertions over the pure
  `Range` type: `from_str` parsing (`"12-18"`, single value, garbage → 49 default),
  `FromStr` trait parity with the inherent `from_str`, inclusive `in_range` bounds, and
  `get_tag_and_range` split/trim + tag-only default.

**Verification:**
- `cargo test -p spf_core` → 9 passed, 0 failed (7 new + 2 pre-existing).
- `cargo fmt -p spf_core -- --check` → clean.
- `cargo clippy` → no new warnings from the test code.

---

## Stage T2 — Pure-logic unit tests (highest ROI) 🟡

**Goal:** table-driven `#[cfg(test)]` coverage of the pure hotspots, colocated in their
modules. No I/O, no external deps.

**Scope decision (this pass):** cover the **`spf_core`** pure targets only, and treat the
`OffensiveBox::from_str` alias defects as a **test-driven bugfix** (fix obvious typos, then
assert corrected behavior). The `spf`-crate `resulthandler.rs` target is **deferred** to a
follow-up (it needs `GameState`/`PlayResult` builders and is the natural pairing for T3 /
WS Stage 2).

**Targets (this pass — `spf_core`):**
- `lineup.rs` — legality checks (`is_legal_lineup` (both impls), `validate_count`,
  `count_array_spots`), plus `from_str` for `OffensiveBox` / `DefensiveBox`.
  - **Bugfix:** correct obvious malformed alias literals in `OffensiveBox::from_str`
    (e.g. the trailing-space `"fl1 "` on line 57, which makes the intended `fl1` input
    unmatchable). Audit callers first; fix only unambiguous typos, surfacing anything
    ambiguous rather than guessing. Each fix pairs with an asserting test.
- `players.rs` — `TeamID::create_from_str` fixup cases (the hardcoded name-normalization
  table) + the `splitn` year/name defaults.
- `stats.rs` — extend beyond `Range` to `RangedStats` where feasible (valid + malformed
  input, no fixtures).

**Deferred (T2 remainder):**
- `resulthandler.rs` — `calculate_play_result` state transitions (down advance, first-down
  reset, possession change, scoring). First `spf`-crate tests; carried forward.
- `is_legal_lineup` (both impls) — full legality needs `Standard*Lineup`/`Roster` builders
  (real `Player` stats), which is fixture-ish and out of scope for this pure pass. The
  underlying `LineupUtilities` count/validation helpers **are** covered directly.

**What landed:**
- **Bugfix** in `spf_core/src/lineup.rs`: `OffensiveBox::from_str` alias `"fl1 "` (trailing
  space, unmatchable — dead code) → `"fl1"`. Caller audit confirmed the only runtime callers
  are in `spf/src/game/fac.rs` parsing `fac_cards.csv`; that data uses `FL` (never `FL1`),
  so the run-direction path was unaffected and the fix only *adds* the intended `fl1` alias.
  No other alias was ambiguous, so none other was touched.
- `lineup.rs` — 10 tests: `OffensiveBox`/`DefensiveBox` `from_str` (full alias maps,
  case-insensitivity, error paths incl. the stale `"fl1 "` form) and `LineupUtilities`
  `validate_count` (inclusive bounds), `count_spots`, `count_array_spots` (sum + over-max).
- `players.rs` — 6 `TeamID::create_from_str` tests (fixup table, unmapped pass-through,
  multi-word names, trimming, missing-name default).
- `stats.rs` — 3 `RangedStats<PassResult>` tests (`create_from_strs`, `get_category` with and
  without boundary shift, and that an unparseable tag is skipped during construction).

**Characterization note (no code change):** `TeamID::create_from_str("")` yields
`year == ""` (not the `"1980"` default) because `splitn(2, ' ')` always emits one element for
a trimmed-empty string; only `name` falls back (`"Omaha"`). Test documents *current*
behavior. Empty team input is not real data, and the `"1980"` default is not a typo, so it
was left as-is rather than changed under a testing task.

**Verification:**
- `cargo test --workspace` → 28 passed, 0 failed (9 pre-existing + 19 new; round-trip test
  still passes / self-skips as before, unaffected by the alias fix).
- `cargo fmt -p spf_core -- --check` → clean.
- `cargo clippy -p spf_core --tests` → no new warnings from the test code (remaining lints
  are the pre-existing workspace noise slated for T6).

---

## Stage T3 — Deterministic FAC seam ⬜

**Goal:** make card-draw-dependent logic testable by injecting a known deck.

- Introduce `FacManager::from_cards(...)` (or `with_deck(...)`) that builds a deck from an
  explicit ordered list, bypassing `thread_rng()` (see `../design/testing-strategy.md` §5).
- Add a focused test asserting a known deck yields a known `PlayResult`.

**Checkpoint:** a deterministic play-execution test passes repeatably.

**Natural pairing:** WS Stage 2 (`Game` emits events) needs a deterministic
"run play → assert event" test, which this unblocks.

---

## Stage T4 — HTTP / integration tests ⬜

**Goal:** exercise the server through its real route wiring.

- Create `spf/tests/`. Use actix `test::init_service(App::new()…)` against the actual App.
- Provide test data: load a tiny fixture league from disk (self-skipping when absent) or add
  a `TeamList`/`Roster` builder.
- Cover representative flows: `POST /game/start`, `POST /game/play`, lineup set/get, and
  error paths (`409` no game, `404` unknown team).
- Add `[dev-dependencies]` only as needed (e.g. `futures-util` for stream helpers).

**Checkpoint:** integration tests pass; error paths asserted.

**Natural pairing:** WS Stage 3's end-to-end test (connect a WS client, assert a frame
arrives) lands here.

---

## Stage T5 — CI gate ⬜

**Goal:** lock in the investment with automation.

- Add `.github/workflows/ci.yml` running, on push/PR:
  - `cargo fmt -- --check`
  - `cargo clippy --workspace -- -D warnings` (enable `-D warnings` after T6)
  - `cargo test --workspace`

**Checkpoint:** CI is green on `main` and required on PRs.

---

## Stage T6 — Warning cleanup ⬜

**Goal:** raise signal quality and enable a `-D warnings` clippy gate.

- Resolve the build warnings (largely dead code / unused imports across the workspace).
- Separate from testing per se, but a prerequisite for the strict clippy step in T5.

**Checkpoint:** `cargo build --workspace` and `cargo clippy --workspace` emit no warnings.

---

## Relationship to the WebSocket-events work

The WS feature is parked (its plan is in `ws-events-*.md`), but the intended interplay is:

| WS stage | Testing tie-in |
|---|---|
| WS Stage 1 (deps + `GameEvent` type) | Inert type; nothing meaningful to unit-test beyond "it compiles". |
| WS Stage 2 (`Game` emits events) | First testable behavior. Depends on **T3** (FAC seam) for a deterministic "run play → assert event" test. |
| WS Stage 3 (WS transport) | Add the end-to-end connect/receive test under **T4** (`spf/tests/`). |

**Recommendation of record:** when WS work resumes, do WS Stage 1 without new tests
(nothing to assert), then land **T3** alongside WS Stage 2. T2/T4/T5/T6 can proceed
independently as capacity allows.
