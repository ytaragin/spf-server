# Stage 3 — WebSocket Transport: `GET /game/ws` (detailed tasks)

> **Status: ✅ Done.** All tasks landed. `cargo build -p spf`, `cargo clippy -p spf` (209
> baseline warnings, none from the new code), `cargo fmt -- --check`, and
> `cargo test --workspace` (48 tests) all pass. Manual smoke test confirmed: WS upgrade
> returns `101 Switching Protocols`, the client receives the `GameStarted` snapshot frame on
> connect, and `409 Conflict` is returned when no game is in progress.
>
> **Deviation from the tasks below (registration).** `game_ws` is **not** registered inside
> the utoipa `scope::scope("/game")`. Two constraints forced this:
> 1. utoipa's scope/`service()` bound is `HttpServiceFactory + OpenApiFactory`. utoipa only
>    implements `OpenApiFactory` for `#[utoipa::path]`-annotated handlers, and a WebSocket
>    upgrade cannot be described by `#[utoipa::path]` (D6). So `game_ws` cannot be a utoipa
>    scope `service`.
> 2. Registering it *after* `split_for_parts()` on the plain `App` (an earlier attempt)
>    compiled but returned **404**: the utoipa `/game` scope, registered first, greedily
>    matches the `/game/*` prefix and has no `/ws` child, so it 404s before the later
>    registration is consulted.
>
> **Final approach:** register on the `UtoipaApp` *before* the scopes using
> `.route("/game/ws", web::get().to(game_ws))` — `UtoipaApp::route` takes a plain
> `actix_web::Route` and carries **no** `OpenApiFactory` bound. `game_ws` is therefore a
> plain `async fn` (no `#[get]`/`#[utoipa::path]` attribute). This keeps it in the utoipa
> app's routing tree, ordered ahead of the `/game` scope so the path resolves. See Task 3's
> updated note.
>
> Concrete, file-level task list for **Stage 3** of the WebSocket events rollout
> (`ws-events-stages.md`). Stage 3 exposes the first *transport adapter*: a read-only
> WebSocket endpoint that, on connect, pushes the current game state (snapshot) and then
> streams every subsequent `GameEvent` as a JSON text frame. Client commands stay on REST.

Prerequisite reading: `../../design/ws-events-architecture.md` (§2 layering, §5 transport
adapters) and `ws-events-stage2.md` (the `Game` emitter — `subscribe()` now exists and is
consumed here).

---

## Outcome / definition of done

- A `game_ws` handler in `spf/src/webendpoint.rs` upgrades an HTTP request to a WebSocket,
  returning the existing `409 Conflict` when no game is in progress.
- On connect the client immediately receives a **snapshot** frame (current `GameState`,
  shaped as `GameEvent::GameStarted { state }`), then every subsequent `GameEvent` as a
  JSON text frame.
- The connection answers `Ping` with `Pong` and exits cleanly on `Close`, channel closure,
  or client-stream end. A lagging receiver (`RecvError::Lagged`) skips and resynchronizes
  rather than tearing down.
- The route is registered at **`GET /game/ws`** (nested under the existing `/game` scope).
- The temporary `#[allow(dead_code)]` on `Game::subscribe()` (added in Stage 2) is
  **removed** — the WS handler now consumes it.
- `cargo build -p spf`, `cargo clippy -p spf`, `cargo fmt -- --check`, and
  `cargo test --workspace` all pass. The stage's functional proof is a **manual** smoke
  test (no new automated test — see Task 4).

**Scope guard:** the only files touched are `spf/src/webendpoint.rs` (handler + route
registration) and `spf/src/game.rs` (one-line lint removal). **No** OpenAPI schema
registration, **no** `AGENTS.md`/Postman edits, **no** changes to `Game`'s emitter or
`events.rs`. Those are Stage 4.

---

## Prerequisites (all confirmed present)

- Dependencies `actix-ws 0.4`, `tokio` (features `sync`, `rt`, `macros`), and
  `futures-util 0.3` are already in `spf/Cargo.toml` (added in Stage 1). **No new deps.**
- `Game::subscribe(&self) -> broadcast::Receiver<GameEvent>` exists (`game.rs:264`),
  currently carrying `#[allow(dead_code)]` — this stage consumes it (Task 5).
- The `409 Conflict` "no game in progress" convention exists (the `lock_game!` macro,
  `webendpoint.rs:26`).
- CORS already permits `GET` from any origin (`runserver`, `webendpoint.rs:533`) — no CORS
  change is required for the WS upgrade.

---

## Design decisions locked for this stage

### D1 — Snapshot frame shape: reuse `GameEvent::GameStarted { state }`

Snapshot-on-connect is **confirmed** for Stage 3 (`ws-events-stages.md`). The initial state
is delivered by serializing `GameEvent::GameStarted { state: <current game.state> }` and
sending it as the first text frame. This means **every** message on the socket is a
uniformly tagged `GameEvent` (`{ "event": "...", "data": {...} }`), so the client parses one
shape. It requires **no** new variant and **no** `events.rs` change (keeping Stage 3 to the
two files above). A distinct `Snapshot` variant was considered and rejected for this stage:
it would touch `events.rs` (out of scope) for no client-side benefit.

### D2 — Grab the receiver *and* snapshot under the lock, then release before the pump

The handler locks `appstate.game` **once**, briefly, to read `game.state` (the snapshot)
and call `game.subscribe()` (the receiver), then drops the guard *before* doing any async
WS work. Rationale: the per-connection pump is a long-lived async task; it must **not** hold
the `Mutex<Option<Game>>` guard (which is `!Send` across await points and would serialize
all game access behind one socket). `GameState` is `Copy` and the `broadcast::Receiver` is
owned, so both survive after the guard drops.

> The existing `lock_game!` macro is **not** reused here: it early-returns `impl Responder`,
> but `game_ws` returns `Result<HttpResponse, actix_web::Error>` (required by
> `actix_ws::handle`). The lock/`409` is handled inline instead.

### D3 — Two event sources multiplexed with `tokio::select!`

The pump loops over a `tokio::select!` on:
1. **`rx.recv()`** — the broadcast stream of `GameEvent`s.
2. **`msg_stream.next()`** — inbound WS frames from the client.

This lets the task both forward server events and respond to client `Ping`/`Close`
promptly, without one blocking the other.

### D4 — Lag handling: skip and continue (do not tear down)

On `Err(RecvError::Lagged(n))`, the pump logs (optional) and **continues** — the client
resynchronizes from subsequent events. With capacity `128` (Stage 2 `GAME_EVENT_CHANNEL_
CAPACITY`) and the initial snapshot already delivered, dropping a few intermediate events is
acceptable for this stage. Re-sending a fresh snapshot after lag is **explicitly deferred**
(a future refinement, not Stage 3). On `Err(RecvError::Closed)` (game dropped/replaced) the
pump breaks and closes the socket.

### D5 — Read-only endpoint

The WebSocket is server → client only. Inbound frames are handled minimally: `Ping` →
`Pong`, `Close`/stream-end → break. All other inbound frame types (`Text`, `Binary`,
`Continuation`) are ignored — commands remain on REST (`ws-events-stages.md` guiding
constraints).

### D6 — No OpenAPI annotation in this stage

`game_ws` intentionally has **no** `#[utoipa::path]` attribute: utoipa cannot describe
WebSocket upgrades, and schema/endpoint documentation is Stage 4. The endpoint simply will
not appear in the generated spec yet — this is expected, not an omission to fix.

---

## Task 1 — Imports (`spf/src/webendpoint.rs`)

Extend the existing `actix_web` import and add the WS/stream/broadcast imports near the top
of the file:

```rust
use actix_web::{get, http::header, post, rt, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_ws::Message;
use futures_util::StreamExt;
use tokio::sync::broadcast::error::RecvError;

use crate::game::events::GameEvent;
```

Verify during implementation that `events` is declared `pub mod events;` in `game.rs` so
`crate::game::events::GameEvent` is reachable from `webendpoint.rs`. (If it is not `pub`,
either make it `pub` or re-export `GameEvent` from `crate::game`, mirroring how `Game`,
`GameState`, `PlayAndState` are already re-exported there — prefer the re-export to keep the
module boundary tidy.)

---

## Task 2 — The `game_ws` handler (`spf/src/webendpoint.rs`)

Add the handler (place it near the other `/game`-scope handlers). Note the return type is
`Result<HttpResponse, actix_web::Error>`, **not** `impl Responder`.

```rust
#[get("/ws")]
async fn game_ws(
    req: HttpRequest,
    body: web::Payload,
    appstate: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    // 1. Briefly lock: read the snapshot and mint a receiver, then release (D2).
    let (snapshot, mut rx) = {
        let mut guard = appstate.game.lock().unwrap();
        let game = match guard.as_mut() {
            Some(g) => g,
            None => return Ok(HttpResponse::Conflict().body("No game in progress")),
        };
        (game.state, game.subscribe())
    };

    // 2. Upgrade the connection.
    let (res, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    // 3. Spawn the per-connection pump.
    rt::spawn(async move {
        // 3a. Snapshot-then-stream (D1): send the current state first.
        let snapshot_ev = GameEvent::GameStarted { state: snapshot };
        if let Ok(txt) = serde_json::to_string(&snapshot_ev) {
            if session.text(txt).await.is_err() {
                return; // client already gone
            }
        }

        // 3b. Multiplex broadcast events and inbound client frames (D3).
        loop {
            tokio::select! {
                event = rx.recv() => match event {
                    Ok(ev) => {
                        if let Ok(txt) = serde_json::to_string(&ev) {
                            if session.text(txt).await.is_err() {
                                break; // client disconnected mid-send
                            }
                        }
                    }
                    Err(RecvError::Lagged(_)) => continue, // skip & resync (D4)
                    Err(RecvError::Closed) => break,       // game dropped (D4)
                },
                msg = msg_stream.next() => match msg {
                    Some(Ok(Message::Ping(bytes))) => {
                        if session.pong(&bytes).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break, // client closed / stream ended
                    Some(Ok(_)) => {} // read-only: ignore Text/Binary/etc. (D5)
                    Some(Err(_)) => break, // protocol error
                },
            }
        }

        let _ = session.close(None).await;
    });

    // 4. Hand the upgrade response back to actix.
    Ok(res)
}
```

Notes:
- `web::Payload` is the raw body stream `actix_ws::handle` needs; it is already available
  via the existing `actix_web::web` import.
- `session` is moved into the spawned task; each `.text()`/`.pong()`/`.close()` is awaited.
- The `msg_stream` yields `actix_ws::Message` items (use the non-aggregated stream — no
  need to aggregate continuation frames for this read-only endpoint).

---

## Task 3 — Register the route (`spf/src/webendpoint.rs`)

> **Landed differently — see the status note at the top.** `game_ws` is a plain `async fn`
> (no `#[get]`/`#[utoipa::path]`) registered on the `UtoipaApp` **before** the scopes via
> `UtoipaApp::route`, which takes an `actix_web::Route` and has no `OpenApiFactory` bound:
>
> ```rust
> let (app, api) = App::new()
>     .into_utoipa_app()
>     .openapi(ApiDoc::openapi())
>     .app_data(app_state.clone())
>     .map(|a| a.wrap(cors))
>     .route("/game/ws", web::get().to(game_ws)) // <-- before the /game scope
>     .service(scope::scope("/game") /* ... */)
>     // ... other scopes ...
>     .split_for_parts();
>
> app.service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", api.clone()))
> ```
>
> Ordering matters: the `/game/ws` route must be registered ahead of the utoipa
> `scope::scope("/game")`, otherwise that scope shadows the path and returns 404. This
> produces `GET /game/ws`. No CORS change. Verified via a live upgrade returning `101`.

Original intent — add `game_ws` to the existing `/game` scope, giving `GET /game/ws`:

```rust
scope::scope("/game")
    .service(start_game)
    .service(get_game_state)
    .service(run_play)
    .service(get_all_plays)
    .service(save_game)
    .service(get_next_play_types)
    .service(set_next_play_type)
    .service(game_ws), // <-- new
```

No CORS change (D-prereqs). No change elsewhere in `runserver`.

---

## Task 4 — Remove the Stage 2 `#[allow(dead_code)]` on `subscribe()` (`spf/src/game.rs`)

`Game::subscribe()` (`game.rs:264`) is now called by `game_ws`, so the unused-method lint no
longer fires. Delete the temporary attribute:

```rust
// DELETE this line above `pub fn subscribe(...)`:
#[allow(dead_code)] // consumed by the Stage 3 WS handler; remove then.
```

Leave the doc comment and signature intact. (If clippy still flags anything, revisit — but
the single call site in `game_ws` makes it live.)

---

## Files touched in Stage 3

| File | Change |
|---|---|
| `spf/src/webendpoint.rs` | WS/stream/broadcast imports; `game_ws` handler (inline lock/`409`, snapshot-then-stream pump, `Ping`→`Pong`, lag-skip, clean close); register `game_ws` under the `/game` scope |
| `spf/src/game.rs` | Remove the temporary `#[allow(dead_code)]` on `subscribe()` (possibly re-export `GameEvent` from `crate::game` — see Task 1) |

Explicitly **not** touched: `events.rs`, `main.rs`, `Cargo.toml` (no new deps), `engine.rs`,
the `ApiDoc` schema list, `AGENTS.md`, the Postman collection.

---

## Verification checklist

1. `cargo build -p spf` — compiles; no new deps.
2. `cargo clippy -p spf` — clean. In particular, the `subscribe()` unused-method warning is
   gone (it is now called), and the new handler introduces no warnings.
3. `cargo fmt -- --check` — formatting matches.
4. `cargo test --workspace` — the existing 42 tests still pass (no new automated test this
   stage).
5. **Manual smoke test** (the stage's functional proof, per `ws-events-stages.md`):
   - `cargo run -p spf`
   - `POST /game/start` with a valid home/away.
   - Connect a WS client: `websocat ws://127.0.0.1:8080/game/ws`. Expect an immediate
     `GameStarted` snapshot frame carrying the current `GameState`.
   - Drive state via REST: `POST /game/nexttype`, then `POST /game/play`. Confirm
     `NextPlayTypeSet` and `PlayRun` JSON frames arrive over the socket (D3 ordering:
     `NextPlayTypeSet` then `PlayRun` for a single play — see stage 2 D3).
   - Connecting the WS client **before** starting a game returns `409 Conflict`.

---

## Risks & gotchas

- **Holding the lock across `await`.** Do not keep the `Mutex` guard alive into the spawned
  task (it is `!Send` and would serialize all game access). Read the snapshot + mint the
  receiver in a tight scoped block, then drop the guard (D2). The compiler will typically
  reject the `!Send` guard-across-await, but scope it explicitly regardless.
- **Return type mismatch.** `actix_ws::handle` requires the handler to return a real
  `HttpResponse`/`Result<HttpResponse, Error>`; the `lock_game!` macro (built for
  `impl Responder`) cannot be used here. Handle the lock/`409` inline.
- **`GameEvent` reachability.** If `crate::game::events` is not `pub`, add a re-export of
  `GameEvent` from `crate::game` rather than widening the module — mirrors the existing
  `Game`/`GameState` re-exports (Task 1).
- **Lag under bursty producers.** `128` capacity is sized for today's max burst of 2
  (stage 2 D1). If future events make bursts larger, a slow client could `Lagged`-skip; the
  snapshot-after-lag refinement is deferred (D4).
- **No OpenAPI entry yet.** `game_ws` deliberately lacks `#[utoipa::path]` (D6); do not add
  it here — that is Stage 4.

---

## Notes carried forward to Stage 4

- Register `GameEvent` (and any inner types not already in `ApiDoc`) in the
  `components(schemas(...))` list, with a `///` note describing the `GET /game/ws` endpoint
  (utoipa cannot natively describe WebSockets).
- Update `AGENTS.md` and, optionally, `spf.postman_collection.json` to mention the WS
  endpoint and the `GameEvent` frame shape.
- Final `cargo fmt`, `cargo clippy`, `cargo build` pass.
