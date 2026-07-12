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
  general fix. (See `docs/plans/completed/ws-events-stage2.md` → Task 6 note.)

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

## 2. No automated test for the WebSocket transport (`GET /game/ws`)

**Status:** open.

### Problem

WS Stage 3 (`docs/plans/completed/ws-events-stage3.md`) shipped `game_ws` with only a **manual**
smoke test (`websocat`, verified once during implementation). There is no automated test
that asserts the handshake, the snapshot-on-connect frame, event forwarding, or the `409`
when no game is in progress. A regression here (e.g. someone reorders route registration
and the `/game` scope shadows `/game/ws` again — see item 4 below) would not be caught by
`cargo test --workspace`.

### Where it shows up today

- `spf/src/webendpoint.rs` — `game_ws` handler has zero test coverage.

### Why it matters

- The registration-order bug we hit during Stage 3 (utoipa's `/game` scope shadowing
  `/game/ws` with a 404) is exactly the kind of regression a test would catch immediately.
- As more transports/events are added (Stage 4+, or new event variants), an automated
  contract test would give confidence without a manual `websocat` session each time.

### Sketch of a more elegant approach (to design later)

- Use `actix_web::test` with a real (or minimal) app builder plus an actix WS test client
  (`actix_ws` supports server-side testing via `actix_web::test::TestRequest` +
  `actix_web::test::call_service`, or a small `awc`-based WS client) to: start a game, open
  `/game/ws`, assert the first frame is `GameStarted` with the expected state, drive a
  REST mutation, and assert the corresponding event frame arrives.
- At minimum, add a lighter-weight test that exercises the route registration/ordering
  (e.g. asserts `/game/ws` returns `409`, not `404`, when no game is running) to guard
  against the specific shadowing regression.

---

## 3. No resync after a lagged WebSocket client (`RecvError::Lagged`)

**Status:** open.

### Problem

The `game_ws` pump (`spf/src/webendpoint.rs`) treats `RecvError::Lagged(n)` as "skip and
continue" (see `docs/plans/completed/ws-events-stage3.md` D4): the client silently misses `n` events and only
resumes receiving from whatever event comes next. There is no mechanism to bring a lagged
client back in sync with a fresh snapshot.

### Where it shows up today

- `game_ws`'s `tokio::select!` loop: `Err(RecvError::Lagged(_)) => continue`.

### Why it matters

- A client that lags (slow network, brief disconnect, GC pause) can end up silently missing
  state-changing events (e.g. a lineup set or a play result) with no signal that it happened
  and no way to recover other than a full reconnect.
- Channel capacity (`GAME_EVENT_CHANNEL_CAPACITY = 128`) makes this unlikely under normal
  single-play/single-lineup bursts, but it is not impossible, especially as more event
  variants are added.

### Sketch of a more elegant approach (to design later)

- On `Lagged(n)`, re-send a snapshot frame (`GameEvent::GameStarted { state: <current
  game.state> }`) before resuming the stream, so the client can always recover full state
  even after missing intermediate events. Requires re-locking the game briefly inside the
  pump task (mirroring the connect-time snapshot logic) — a small, self-contained change,
  but deferred until this is prioritized.
- Alternatively (or additionally), emit a distinct `Lagged`/`Resynced` event so the client
  can visibly log/handle the gap rather than silently continuing.

---

## 4. `utoipa` scope vs. plain-route registration is a subtle, undocumented gotcha

**Status:** open (partially mitigated by inline comments + `docs/plans/completed/ws-events-stage3.md`).

### Problem

`utoipa-actix-web`'s `Scope::service()`/`UtoipaApp::service()` require the service to
implement `OpenApiFactory`, which utoipa only provides for `#[utoipa::path]`-annotated
handlers. Any handler that *cannot* carry `#[utoipa::path]` (WebSocket upgrades today; any
future streaming/non-REST handler tomorrow) cannot be added to a utoipa scope at all, and
must instead use `UtoipaApp::route(path, Route)` (which has no such bound) — registered
**before** any overlapping scope, or that scope will shadow the path and return a `404`
instead of ever reaching the handler.

This is documented today only as inline comments on `game_ws` and in
`docs/plans/completed/ws-events-stage3.md`'s deviation note — not in the durable
`docs/design/openapi-utoipa.md` reference, where a future contributor adding a similar
non-REST handler would be more likely to look.

### Where it shows up today

- `spf/src/webendpoint.rs`: `game_ws` registered via
  `.route("/game/ws", web::get().to(game_ws))` ahead of `scope::scope("/game")`.
- `docs/plans/completed/ws-events-stage3.md` (Task 3's "landed differently" note).

### Why it matters

- The failure mode (silent 404, not a compile error) is confusing: the code compiles fine
  either way, and the ordering requirement is not enforced by the type system.
- The next person adding a second non-REST endpoint (e.g. an SSE stream, a raw file
  upload) is likely to hit the exact same two-step trap (compile error inside a scope, then
  a 404 after moving it out) without this write-up.

### Sketch of a more elegant approach (to design later)

- Add a short subsection to `docs/design/openapi-utoipa.md` documenting: (a) the
  `OpenApiFactory` bound and why WS/streaming handlers can't satisfy it, (b) the
  `UtoipaApp::route` escape hatch, and (c) the registration-order requirement relative to
  overlapping scopes. This turns a "discovered the hard way" gotcha into a durable reference
  so it isn't rediscovered per-feature.

---
