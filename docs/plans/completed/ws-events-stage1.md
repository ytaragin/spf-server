# Stage 1 — Foundation: Dependencies & Event Type (detailed tasks)

> **Status: ✅ Done.** All three files landed as specified below. `cargo build`,
> `cargo clippy -p spf` (no new warnings), and `cargo fmt -- --check` all pass. This doc is
> retained as the record of what shipped; the next stage is `ws-events-stage2.md`.

Concrete, file-level task list for **Stage 1** of the WebSocket events rollout
(`ws-events-stages.md`). Stage 1 introduces the crate dependencies and defines the
`GameEvent` type. **No behaviour changes** — nothing emits or consumes events yet.

Prerequisite reading: `../../design/ws-events-architecture.md` (§3 explains why `GameEvent` must be
`Clone + Serialize`).

---

## Outcome / definition of done — ✅ all met

- ✅ `spf/Cargo.toml` declares `actix-ws`, `tokio`, and `futures-util`.
- ✅ `spf/src/game/events.rs` exists and defines the `GameEvent` enum.
- ✅ `game.rs` registers the `events` module.
- ✅ `cargo build` and `cargo clippy` pass. `GameEvent` compiles but is intentionally unused
  (a `#[allow(dead_code)]` on the enum keeps clippy quiet until Stage 2 wires it in).

---

## Task 1 — Add dependencies (`spf/Cargo.toml`)

Add to the `[dependencies]` table:

```toml
actix-ws = "0.4"
tokio = { version = "1", features = ["sync", "rt", "macros"] }
futures-util = "0.3"
```

Notes:
- `tokio 1.49` and `futures-util` are already present **transitively** via actix (verified
  with `cargo tree`), so declaring them as direct deps does not introduce new major
  versions — it only makes them directly importable.
- `tokio` features needed: `sync` (for `broadcast`, used in Stage 2), `rt` + `macros` (for
  `tokio::select!` in the Stage 3 WS pump). Declaring them now avoids a second edit.
- `actix-ws` and `futures-util` are not *used* until Stage 3, but adding all three deps in
  one stage keeps `Cargo.toml`/`Cargo.lock` churn in a single reviewable step. (If you
  prefer strict just-in-time deps, `actix-ws` + `futures-util` may be deferred to Stage 3;
  `tokio`'s `sync` feature is the only one Stage 2 strictly needs.)

**Verify:** `cargo build` (this resolves and locks the new deps).

---

## Task 2 — Create the event type (`spf/src/game/events.rs`)

New file. Defines the single extensible event enum.

### Types the enum references (all already exist and are serializable)

| Referenced type | Defined in | Notes |
|---|---|---|
| `GameState` | `game.rs` | `#[derive(… Serialize … ToSchema)]` already ✔ |
| `PlayAndState` | `game.rs` | already `Serialize + ToSchema`; carries `result` + `new_state` ✔ |
| `OffenseIDLineup` | `game/engine.rs` | already `Serialize + ToSchema` (untagged) ✔ |
| `DefenseIDLineup` | `game/engine.rs` | already `Serialize + ToSchema` (untagged) ✔ |
| `PlayType` | `game/engine.rs` | already `Serialize + ToSchema` ✔ |

No changes to those types are required in Stage 1.

### Proposed content

```rust
//! Domain events emitted by [`Game`](crate::game::Game) when its state changes.
//!
//! These are published on a broadcast channel and consumed by transport adapters
//! (currently the WebSocket handler). See `docs/design/ws-events-architecture.md`.

use serde::Serialize;
use utoipa::ToSchema;

use crate::game::{
    engine::{DefenseIDLineup, OffenseIDLineup, PlayType},
    GameState, PlayAndState,
};

/// An event describing something that happened to the game.
///
/// Serialized form is a tagged object: `{ "event": "<Variant>", "data": { … } }`.
/// The set of variants is expected to grow; adding one is an additive change plus a
/// single `emit()` call at the point the event occurs (Stage 2+).
#[allow(dead_code)] // wired up in Stage 2; remove once `emit()`/`subscribe()` exist.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(tag = "event", content = "data")]
pub enum GameEvent {
    /// A new game was created.
    GameStarted { state: GameState },

    /// The offensive lineup was set for the upcoming play.
    OffensiveLineupSet { lineup: OffenseIDLineup },

    /// The defensive lineup was set for the upcoming play.
    DefensiveLineupSet { lineup: DefenseIDLineup },

    /// The next play type was selected.
    NextPlayTypeSet { play_type: PlayType },

    /// A play was executed; carries the result and the resulting game state.
    ///
    /// Boxed because `PlayAndState` is significantly larger than the other variants
    /// (avoids bloating the enum's size for every event).
    PlayRun { play: Box<PlayAndState> },
    // Future variants go here (e.g. penalties, clock/quarter changes, calls-set once
    // OffenseCall/DefenseCall are serializable).
}
```

Design notes:
- **`#[serde(tag = "event", content = "data")]`** gives clients a stable, discriminated
  shape: `{"event":"PlayRun","data":{"play":{…}}}`. Easy to `switch`/match on the client.
- **`Clone`** is required by the broadcast channel; **`Serialize`** by the transports
  (architecture §3).
- **`ToSchema`** lets the payload be documented in OpenAPI in Stage 4.
- **`Box<PlayAndState>`** keeps the enum small (clippy `large_enum_variant`-friendly).
- **`#[allow(dead_code)]`** is temporary — removed in Stage 2 when `emit()`/`subscribe()`
  reference the variants.

---

## Task 3 — Register the module (`spf/src/game.rs`)

`game.rs` currently begins with:

```rust
pub mod engine;
pub mod fac;
pub mod kickoff_play;
pub mod standard_play;
```

Add the new module alongside them:

```rust
pub mod engine;
pub mod events;
pub mod fac;
pub mod kickoff_play;
pub mod standard_play;
```

No other edits to `game.rs` in Stage 1. (The `Game` struct field, `emit()`, `subscribe()`,
and the emission call sites are all **Stage 2**.)

---

## Files touched in Stage 1

| File | Change |
|---|---|
| `spf/Cargo.toml` | Add `actix-ws`, `tokio`, `futures-util` dependencies |
| `spf/src/game/events.rs` | **New** — define `GameEvent` enum |
| `spf/src/game.rs` | Add `pub mod events;` |

Explicitly **not** touched in Stage 1: `webendpoint.rs`, `main.rs`, the `Game` struct
body, any handler.

---

## Verification checklist — ✅ all passed

1. ✅ `cargo build` — compiled, new deps resolved/locked (`actix-ws 0.4.0`, `tokio 1.49.0`,
   `futures-util 0.3.31`; no new major versions).
2. ✅ `cargo clippy` — clean (the `#[allow(dead_code)]` suppresses the unused-enum warning;
   `GameEvent` did not trip `large_enum_variant` thanks to the boxed `PlayRun`).
3. ✅ `cargo fmt -- --check` — formatting matches repo style.
4. ✅ Sanity: `GameEvent` is importable, e.g. `use crate::game::events::GameEvent;` resolves.

No runtime behaviour changes; the server starts and serves exactly as before.

---

## Notes carried forward to Stage 2

- Remove the `#[allow(dead_code)]` once `emit()`/`subscribe()` reference the variants.
- The channel capacity for `broadcast::channel(N)` is chosen in Stage 2 (a small constant,
  e.g. 64–256, sized for expected event bursts vs. slow-client lag tolerance).
