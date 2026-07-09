# WebSocket Events: Staged Rollout Plan

High-level plan for delivering the event-broadcast feature described in
`../design/ws-events-architecture.md`. The work is split into independent stages that each end at a
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

| Stage | Theme | Outcome |
|---|---|---|
| 1 | Foundation: deps + event type | `GameEvent` enum exists and compiles; dependencies added. Nothing wired yet. |
| 2 | Domain emitter | `Game` owns a broadcast `Sender`, emits on every state change, and exposes `subscribe()`. Verifiable via a unit test. |
| 3 | WebSocket transport | `GET /game/ws` streams live events to connected clients. End-to-end manual smoke test passes. |
| 4 | Docs & polish | `GameEvent` documented in OpenAPI; `AGENTS.md`/Postman updated; final lint/format pass. |

---

## Stage 1 — Foundation: dependencies & event type

**Goal:** introduce the plumbing crate dependencies and define the `GameEvent` type, with
no behavioural change.

- Add `actix-ws`, `tokio` (sync/rt/macros features), and `futures-util` to
  `spf/Cargo.toml`.
- Create `spf/src/game/events.rs` defining the `GameEvent` enum (serializable, cloneable,
  `ToSchema`) with the initial variant set.
- Register the module in `game.rs`.

**Checkpoint:** `cargo build` succeeds; `GameEvent` is referenceable but unused.

*(Concrete file/type-level tasks: see `ws-events-stage1.md`.)*

---

## Stage 2 — Domain emitter: make `Game` publish events

**Goal:** `Game` becomes the event source, per the architecture doc.

- Add the `event_tx: broadcast::Sender<GameEvent>` field (skipped in serialization).
- Build the channel in `create_game`; add `emit()` and `subscribe()`.
- Insert `emit()` calls into the existing mutating methods: `run_current_play`, the
  offensive/defensive lineup setters, `set_next_play_type`, and game creation.

**Checkpoint:** `cargo build` + `cargo clippy`. A unit test subscribes, runs a play, and
asserts the corresponding event is received. No transport yet.

---

## Stage 3 — WebSocket transport: `GET /game/ws`

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

**Decisions to confirm at the start of this stage:**
- WS route path: `GET /game/ws` (preferred) vs. top-level `GET /ws`.
- Snapshot-on-connect: push current `GameState` immediately, or stream future events only.

---

## Stage 4 — Documentation & polish

**Goal:** make the feature discoverable and keep repo conventions intact.

- Add `GameEvent` (and any inner types not already registered) to the `ApiDoc`
  `components(schemas(...))`, with a `///` note describing the WS endpoint (utoipa cannot
  natively describe WebSockets).
- Update `AGENTS.md` (and `spf.postman_collection.json` if desired) to mention the WS
  endpoint and event shapes.
- Final `cargo fmt`, `cargo clippy`, `cargo build`.

**Checkpoint:** docs reflect reality; lints clean.

---

## Deferred (explicitly out of scope for these four stages)

- **Calls-set events** — require deriving `Serialize` on `OffenseCall` / `DefenseCall`
  first.
- **Snapshot-on-connect** — may be folded into Stage 3 if desired; otherwise a later
  additive change.
- **Multiple games / per-game-id topics** — reintroduce a registry keyed by game id; does
  not change the emitter/consumer split.
