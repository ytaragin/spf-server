# WebSocket Events: Staged Rollout Plan

High-level plan for delivering the event-broadcast feature described in
`../../design/ws-events-architecture.md`. The work is split into independent stages that each end at a
compiling, verifiable checkpoint.

Detailed task breakdowns are written **per stage, when we reach that stage**. Stage 1's
detailed tasks live in `ws-events-stage1.md`; the later stages below are intentionally
high level.

---

## Guiding constraints

- **Single game** — keep `AppState.game: Mutex<Option<Game>>`; no game-id routing.
- **Emission in the domain layer** — `Game` emits; REST handlers do not.
- **Read-only WebSocket** — server → client only; commands stay on REST.
- **Additive & incremental** — each stage builds and passes `cargo clippy` on its own; no
  stage leaves the tree broken.

---

## Stage overview

| Stage | Theme | Status | Outcome |
|---|---|---|---|
| 1 | Foundation: deps + event type | ✅ Done | `GameEvent` enum exists and compiles; dependencies added. Nothing wired yet. |
| 2 | Domain emitter | ✅ Done | `Game` owns a broadcast `Sender`, emits on every state change, and exposes `subscribe()`. Verified via a unit test. |
| 3 | WebSocket transport | ✅ Done | `GET /game/ws` streams live events to connected clients (snapshot-then-stream). Verified: build/clippy/fmt/test clean. |
| 4 | Docs & polish | ✅ Done | `GameEvent` registered in the OpenAPI schema (with a description note on the WS endpoint since utoipa can't describe upgrades); `AGENTS.md` updated; final lint/format pass clean. |

**Confirmed decisions (settled during Stage 1 planning, apply at Stage 3):**
- WS route: **`GET /game/ws`** (nested under the existing `/game` scope).
- Snapshot-on-connect: **snapshot then stream** — a newly connected client immediately
  receives the current state, then subsequent events.

---

## Stage 1 — Foundation: dependencies & event type ✅ Done

**Goal:** introduce the plumbing crate dependencies and define the `GameEvent` type, with
no behavioural change.

- Add `actix-ws`, `tokio` (sync/rt/macros features), and `futures-util` to
  `spf/Cargo.toml`.
- Create `spf/src/game/events.rs` defining the `GameEvent` enum (serializable, cloneable,
  `ToSchema`) with the initial variant set.
- Register the module in `game.rs`.

**Checkpoint:** `cargo build` succeeds; `GameEvent` is referenceable but unused.

**What landed:** all three files changed as planned. Deps resolved/locked without new major
versions (`actix-ws 0.4.0`, `tokio 1.49.0`, `futures-util 0.3.31`). `GameEvent` uses
`#[serde(tag = "event", content = "data")]` with `PlayRun { play: Box<PlayAndState> }`
(the `Box` keeps clippy's `large_enum_variant` quiet). A temporary `#[allow(dead_code)]` on
the enum suppresses the unused warning until Stage 2 wires in `emit()`/`subscribe()`.
Verified: `cargo build`, `cargo clippy -p spf` (no new warnings), `cargo fmt -- --check`.

*(Concrete file/type-level tasks: see `ws-events-stage1.md`.)*

---

## Stage 2 — Domain emitter: make `Game` publish events ✅ Done

**Goal:** `Game` becomes the event source, per the architecture doc.

- Add the `event_tx: broadcast::Sender<GameEvent>` field (skipped in serialization).
- Build the channel in `create_game`; add `emit()` and `subscribe()`.
- Insert `emit()` calls into the existing mutating methods: `run_current_play`, the
  offensive/defensive lineup setters, `set_next_play_type`, and game creation.

**Checkpoint:** `cargo build` + `cargo clippy`. A unit test subscribes, drives a state
change, and asserts the corresponding event is received. No transport yet.

**What landed:** all five emission sites wired; `emit()` (private) / `subscribe()` (public,
`#[allow(dead_code)]` until the Stage 3 WS handler consumes it) added; `GAME_EVENT_CHANNEL_CAPACITY = 128`.
The Stage 1 `#[allow(dead_code)]` on `GameEvent` was removed. Verified: `cargo build`,
`cargo clippy -p spf` (89 warnings — one *fewer* than Stage 1's baseline; none from Stage 2),
`cargo fmt -- --check`, `cargo test --workspace` (42 tests incl. the new one).

**One deviation from the detailed plan:** `create_game` loads the FAC deck from a
CWD-relative path (`cards/fac_cards.csv`), and Cargo runs `spf`-crate tests with CWD = the
crate dir, so that path is unreachable and `FacManager::new` would panic. To let the test
actually build a `Game`, `create_game` was refactored to delegate to a private
`create_game_with_fac_path(home, away, fac_path)`; the public API is unchanged. The test
points at `../cards/fac_cards.csv` and self-skips if absent (per testing-strategy §6). This
is a minimal path seam, **not** the full T3 deck-injection work.

*(Concrete file/type-level tasks: see `ws-events-stage2.md`.)*

---

## Stage 3 — WebSocket transport: `GET /game/ws` ✅ Done

**Goal:** expose the first transport adapter.

- Add a `game_ws` handler in `webendpoint.rs`: acquire a receiver via `game.subscribe()`
  (returning the existing `409 Conflict` convention when no game is in progress),
  establish the socket with `actix_ws::handle`, and spawn a per-connection pump task.
- The pump forwards each `GameEvent` as a JSON text frame, replies to `Ping` with `Pong`,
  and exits on `Close`/lag-closed.
- Register the service under the `/game` scope in `runserver`. (CORS already permits `GET`
  and any origin — no change expected.)

**Checkpoint:** manual smoke test — start a game, connect a WS client
(e.g. `websocat ws://127.0.0.1:8080/game/ws`), then run a play / set a lineup via REST and
confirm JSON events arrive.

**Decisions confirmed (settled during Stage 1 planning):**
- WS route path: **`GET /game/ws`** (nested under `/game`), not top-level `GET /ws`.
- Snapshot-on-connect: **snapshot then stream** — push the current state immediately, then
  future events.

**What landed:** `game_ws` handler + `subscribe()` `#[allow(dead_code)]` removed. Snapshot
sent as `GameEvent::GameStarted { state }`; a `tokio::select!` pump multiplexes broadcast
events and inbound client frames (`Ping`→`Pong`, `Lagged`→skip, `Closed`/`Close`→exit).
Verified: `cargo build/clippy/fmt` clean, `cargo test --workspace` (48 tests), and a live
smoke test (WS upgrade → `101`, `GameStarted` snapshot received, `409` with no game).
**Deviation:** `game_ws` is a plain `async fn` registered via
`.route("/game/ws", web::get().to(game_ws))` on the `UtoipaApp` **before** the `/game` scope
— it cannot be a utoipa scope `service` (that bound requires `OpenApiFactory`, which a
non-`#[utoipa::path]` WS handler lacks), and it must precede the `/game` scope or that scope
shadows the path with a 404. Detailed record: `ws-events-stage3.md`.

---

## Stage 4 — Documentation & polish ✅ Done

**Goal:** make the feature discoverable and keep repo conventions intact.

- Add `GameEvent` (and any inner types not already registered) to the `ApiDoc`
  `components(schemas(...))`, with a `///` note describing the WS endpoint (utoipa cannot
  natively describe WebSockets).
- Update `AGENTS.md` (and `spf.postman_collection.json` if desired) to mention the WS
  endpoint and event shapes.
- Final `cargo fmt`, `cargo clippy`, `cargo build`.

**Checkpoint:** docs reflect reality; lints clean.

**What landed:** `GameEvent` added to `ApiDoc`'s `components(schemas(...))` — its inner
types (`GameState`, `PlayAndState`, `OffenseIDLineup`, `DefenseIDLineup`, `PlayType`) were
already reachable via other endpoints' schemas, so no additional types needed registering.
The `ApiDoc` `info.description` gained a paragraph noting the `GET /game/ws` endpoint and
pointing to the README's `websocat` example, since utoipa has no path entry for it.
`AGENTS.md`'s "Quick orientation" section gained a "Live events" bullet linking to
`ws-events-architecture.md` and the README. `spf.postman_collection.json` was **not**
touched — it already predates the current REST routes (e.g. `/getstate` vs. today's
`/game/state`) and updating it was out of scope for this pass. Verified live: started the
server and confirmed `GET /api-docs/openapi.json` contains a `GameEvent` schema and the
updated description. `cargo build/clippy/fmt` clean (209 baseline warnings, unchanged),
`cargo test --workspace` (48 tests) pass.

---

## Deferred (explicitly out of scope for these four stages)

- **Calls-set events** — require deriving `Serialize` on `OffenseCall` / `DefenseCall`
  first.
- **Multiple games / per-game-id topics** — reintroduce a registry keyed by game id; does
  not change the emitter/consumer split.

*(Snapshot-on-connect is no longer deferred — it is confirmed for Stage 3, above.)*
