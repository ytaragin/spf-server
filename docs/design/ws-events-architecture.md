# Architecture: Game Control & Event Flow

Reference document for the event-broadcast architecture that lets external clients
subscribe to real-time game events (lineup set, play result, game lifecycle, …).

This describes the **target design**. Implementation is staged — see
`../plans/ws-events-stages.md` for the rollout plan and `../plans/ws-events-stage1.md` for the first
stage's concrete tasks.

---

## 1. Motivation

Today the server exposes only a REST interface. A REST client must *poll*
(`GET /game/state`, `GET /game/plays`) to learn that something changed. We want a
**push** channel so clients are notified the moment the game changes — a lineup is
set, a play is run, the game starts, etc.

The first transport for this is a WebSocket endpoint, but the design is deliberately
transport-agnostic (see §5).

---

## 2. Layering: who emits, who consumes

The core principle is a **separation between the domain layer and the transport
layer**.

```
  Controllers (input side)                Domain                 Transport (output side)
  ────────────────────────           ───────────────           ─────────────────────────
  REST handlers          ─┐          ┌───────────┐           ┌─► WebSocket connection
  (future) other input    ├─ call ──►│   Game    │─ emit ───►│   (future) SSE stream
  mechanisms             ─┘          │           │  events   │   (future) MQ / Kafka relay
                                     └───────────┘           └─► (future) audit recorder
                                     holds the event
                                     broadcast Sender
```

- **Controllers** (e.g. the existing actix REST handlers) drive the game by calling
  `Game`'s mutating methods. They do **not** emit events.
- **`Game`** is the single source of truth. When one of its methods changes state, that
  method emits a typed `GameEvent`. `Game` knows nothing about HTTP, WebSockets, or any
  client.
- **Transport adapters** subscribe to the game's event stream and forward each event to
  connected clients in whatever form that transport requires (JSON text frame over WS,
  SSE `data:` line, a queue message, a log entry, …).

### Why emission lives in `Game` (not in the REST handlers)

Because emission is bound to the domain method, **any** caller of that method emits the
event — the current REST handler *and* any future controller. If emission lived in the
REST handler, a second input mechanism would silently fail to notify subscribers. Domain
-layer emission guarantees every state change is announced exactly once, regardless of
who triggered it.

---

## 3. The bridge: a broadcast channel

The mechanism connecting `Game` (one producer) to N clients (many consumers) is a
[`tokio::sync::broadcast`](https://docs.rs/tokio/latest/tokio/sync/broadcast/index.html)
channel.

```
                 Game.event_tx.send(ev)          (one send, cloned per receiver)
                          │
        ┌─────────────────┼─────────────────┐
        ▼                 ▼                 ▼
   rx (client A)     rx (client B)     rx (client C)
```

Key properties and why they fit:

| Property | Why it matters here |
|---|---|
| **Multi-producer, multi-consumer, fan-out** | Every active receiver gets its own clone of each event — exactly what "broadcast to all connected clients" needs. (Contrast `mpsc`, where one message goes to exactly one consumer.) |
| **Non-blocking `send`** | `send` does not await and does not block. Emitting an event never stalls the code holding the `Mutex<Game>` lock; the awaiting happens on the transport side, in its own task. |
| **Bounded capacity / lag handling** | The channel has a fixed capacity. A slow consumer that falls behind receives `RecvError::Lagged(n)` (it skips old messages) instead of growing memory without bound or blocking the producer. |
| **Automatic lifecycle** | When a client disconnects, its `Receiver` drops. When the `Game` is replaced/dropped, the `Sender` drops and receivers observe `RecvError::Closed`, cleanly ending their tasks. |

### Consequences for the event type

Because the channel clones per receiver and transports serialize to text, `GameEvent`
must be:

- **`Clone`** — the channel clones the value for each subscriber.
- **`Serialize`** — transports render it (e.g. to a JSON WS frame).

---

## 4. `Game` as an event emitter

`Game` gains one field and two small methods (details/signatures are finalized in the
stage docs):

- **Field** — `event_tx: broadcast::Sender<GameEvent>`, marked `#[serde(skip_serializing)]`
  because it is runtime plumbing, not game data.
- **`emit(&self, ev: GameEvent)`** — internal helper that publishes an event. A send
  error means "no subscribers currently", which is normal and ignored.
- **`subscribe(&self) -> broadcast::Receiver<GameEvent>`** — public; each transport
  adapter calls this to obtain its own receiver.

Emission points are the existing mutating methods, for example:

```rust
pub fn run_current_play(&mut self) -> Result<PlayAndState, String> {
    let res = run_play(/* ... */)?;
    self.past_plays.push(res.clone());
    self.state = GameState { ..res.new_state };
    self.set_next_play_type(self.state.get_next_move_default())?;
    self.emit(GameEvent::PlayRun { play: Box::new(res.clone()) }); // <-- emission here
    Ok(res)
}
```

Other emission points: the lineup setters, the call setters (once those payload types
are serializable — see §6), `set_next_play_type`, and game creation.

---

## 5. Transport adapters (the output side)

A transport adapter is any consumer that:

1. Obtains a receiver via `game.subscribe()`.
2. Loops over incoming events and forwards them however its protocol requires.

The **first** adapter is the WebSocket handler:

- On connect, it briefly locks the game to grab a receiver, then hands off to a
  per-connection task.
- That task forwards each `GameEvent` as a JSON text frame, answers client `Ping` with
  `Pong`, and exits on `Close` or channel closure.
- The WebSocket is **read-only** (server → client). Client commands remain on REST.

### Extensibility guarantee

Adding a *new* transport tomorrow (SSE, a message-queue relay, a recorder, a second WS
variant) requires **no changes to `Game`**. The new adapter simply calls
`game.subscribe()` and consumes events. This is the primary payoff of the layering.

The **only** time `Game` changes for events is when a *new kind of event* is introduced
(a new fact the game announces, e.g. "penalty called"): add a `GameEvent` variant and one
`emit()` call. That is a domain concern, independent of how many transports exist, and
every existing adapter receives the new event automatically.

---

## 6. Scope boundaries & known constraints

- **Single game.** The current server holds one game in
  `AppState.game: Mutex<Option<Game>>`. This architecture keeps that model; there is no
  game-id routing. Multiple concurrent games (a registry keyed by id, per-game topics)
  is a future extension that does not alter the emitter/consumer split described here.
- **Calls-set events are deferred.** `OffenseCall` / `DefenseCall` do not derive
  `Serialize` today, so events carrying them cannot be added until those derives exist.
  All other events (play run, lineups set, lifecycle) use types that are already
  serializable.
- **Snapshot-on-connect is optional.** Whether a newly connected client immediately
  receives the current `GameState` (vs. only future events) is a per-adapter policy
  decision, not a change to the domain layer.

---

## 7. Design alternatives considered

- **Observer/callback list on `Game`** (`Vec<Box<dyn Fn(&GameEvent)>>`). Also decouples
  domain from transport, but it is synchronous (a slow observer blocks the lock holder),
  harder to make `Send`/thread-safe across actix worker threads, and awkward to bridge
  into async transport tasks. `broadcast` is the idiomatic async fit with less code.
- **Emit from REST handlers.** Rejected: a second input mechanism would not emit events,
  breaking the "single source of truth" guarantee. Emission belongs in the domain layer.
