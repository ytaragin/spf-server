# Stage 2 — Domain Emitter: make `Game` publish events (detailed tasks)

> **Status: ✅ Done.** All tasks landed. `cargo build`, `cargo clippy -p spf` (no new
> warnings; one fewer than Stage 1's baseline), `cargo fmt -- --check`, and
> `cargo test --workspace` (42 tests) all pass. **One deviation from the tasks below:**
> `create_game` was refactored to delegate to a private
> `create_game_with_fac_path(home, away, fac_path)` so the unit test can build a `Game`
> without tripping over the CWD-relative FAC-deck path (Cargo runs `spf`-crate tests with
> CWD = crate dir). The public `create_game` API is unchanged; see Task 6's "Building a
> `Game`" note for details.

Concrete, file-level task list for **Stage 2** of the WebSocket events rollout
(`ws-events-stages.md`). Stage 2 turns `Game` into the single event source: it gains a
broadcast `Sender`, a private `emit()` helper, a public `subscribe()`, and `emit()` calls at
each state-mutating method. **Still no transport** — nothing consumes the events yet except
the Stage 2 unit test.

Prerequisite reading: `../../design/ws-events-architecture.md` (§3 the broadcast bridge, §4
`Game` as emitter) and `ws-events-stage1.md` (the `GameEvent` type, now landed).

---

## Outcome / definition of done

- `Game` has a `#[serde(skip_serializing)] event_tx: broadcast::Sender<GameEvent>` field.
- `create_game` builds the channel with a named capacity constant.
- `Game::emit(&self, GameEvent)` (private) and `Game::subscribe(&self) -> Receiver<GameEvent>`
  (public) exist.
- Every state-mutating method emits its corresponding `GameEvent` (table in Task 4).
- The temporary `#[allow(dead_code)]` on `GameEvent` (added in Stage 1) is **removed** — the
  variants are now constructed, so the lint no longer fires.
- A unit test in `game.rs` subscribes and asserts a card-draw-independent event is received.
- `cargo build`, `cargo clippy -p spf`, `cargo fmt -- --check`, and `cargo test -p spf` all
  pass.

**Scope guard:** no `webendpoint.rs` changes, no WS handler, no OpenAPI changes. Those are
Stage 3 / Stage 4. The only files touched are `spf/src/game.rs` and
`spf/src/game/events.rs` (one-line lint removal).

---

## Design decisions locked for this stage

### D1 — Channel capacity: `const GAME_EVENT_CHANNEL_CAPACITY: usize = 128`

`broadcast::channel(N)` requires a bounded capacity. `128` is comfortably above any single
user action's event burst (the largest is `run_current_play`, which emits at most two events
— see D3) while still bounding memory for a slow/absent consumer. A lagging receiver gets
`RecvError::Lagged(n)` and resynchronizes rather than blocking the producer (architecture
§3). Define the constant at module scope in `game.rs` (not a magic literal) with a short
`//` rationale.

### D2 — `GameStarted` is emitted from `create_game`, and that is a deliberate no-op *today*

At the moment `create_game` runs, **no client has subscribed yet** (the WS client subscribes
later, in Stage 3, against the already-created game). `broadcast::Sender::send` with zero
receivers returns `Err(SendError)`, which `emit()` deliberately ignores. So the
`GameStarted` emission has no observable effect in Stage 2.

It is still added now because:
- It keeps the "every mutating method emits" invariant complete and self-documenting.
- The **snapshot-on-connect** behavior confirmed for Stage 3 (push current state on connect)
  is what actually delivers initial state to a client — it does *not* rely on replaying
  `GameStarted`. The two mechanisms are independent; `GameStarted` covers the (future)
  case of a subscriber that is already attached when a *new* game is created.

This nuance is called out so a future reader does not "fix" the seemingly-useless emission.

### D3 — `run_current_play` emits **two** events, in order: `NextPlayTypeSet` then `PlayRun`

`run_current_play` (`game.rs:287`) calls `self.set_next_play_type(...)` internally
(`game.rs:300`) to advance to the next play. Because emission lives in `set_next_play_type`
(so the *direct* REST call to `POST /game/nexttype` also emits — architecture §2's "any
caller emits" guarantee), running a play will naturally emit `NextPlayTypeSet` (from the
internal call) and then `PlayRun` (from `run_current_play` itself, after it pushes the
result and updates state).

This is **correct and intended**: after a play resolves, the allowed next play type genuinely
changed, and a client should learn both facts. The ordering is: emit `PlayRun` *after* the
internal `set_next_play_type`, matching the code's existing sequence (push result → update
state → set next type → return). Document this so the double-emit is understood as designed,
not a bug.

### D4 — Test uses `try_recv()`, not async — **no new dependency**

`broadcast::Sender::send` is synchronous and non-blocking, so immediately after a mutating
method returns, the event is already in the channel's ring buffer.
`broadcast::Receiver::try_recv()` reads it synchronously. Therefore the Stage 2 test needs
**no** `#[tokio::test]`, no async runtime, and **no `[dev-dependencies]`** — consistent with
the project's "no speculative deps" stance (`../../design/testing-strategy.md` §3). The `tokio`
`rt`/`macros` features declared in Stage 1 remain reserved for the Stage 3 WS pump.

### D5 — What the test asserts (determinism)

`GameState`, `GameTeams`, and `GamePlayStatus` do **not** derive `PartialEq`; `PlayType`
**does** (verified: `engine.rs:103`). Following the established pattern in
`resulthandler.rs`'s tests (assert by variant/discriminant, not `assert_eq!` on
non-`PartialEq` types), the primary Stage 2 assertion targets the **card-draw-independent**
`NextPlayTypeSet` event, whose payload (`PlayType`) is directly comparable.

The `PlayRun` event depends on a shuffled FAC deck (`fac.rs` `thread_rng()` — the only
nondeterminism, `../../design/testing-strategy.md` §5), so a deterministic
"run play → assert `PlayRun` contents" test is **out of scope here** and is unlocked by
testing-plan **T3** (the `FacManager::from_cards` seam). Asserting merely that
`run_current_play` *emits some* `PlayRun` (via `matches!`, without inspecting yardage) is
acceptable but secondary; prefer the `NextPlayTypeSet` assertion as the stage's proof.

---

## Task 1 — Imports & capacity constant (`spf/src/game.rs`)

Add the broadcast import alongside the existing `use` items near the top of `game.rs`:

```rust
use tokio::sync::broadcast;

use self::events::GameEvent; // `events` module was registered in Stage 1
```

(True path: `events` is declared `pub mod events;` at the top of `game.rs`, so within the
same module the type is reachable as `events::GameEvent`; add the `use` for brevity at call
sites.)

Add the capacity constant near the other module-level items (above `struct Game`):

```rust
/// Capacity of the per-`Game` event broadcast channel. Sized well above the largest
/// single-action burst (`run_current_play` emits 2 events) so normal use never lags; a
/// slow/absent consumer receives `Lagged` rather than blocking the producer. See
/// docs/design/ws-events-architecture.md §3.
const GAME_EVENT_CHANNEL_CAPACITY: usize = 128;
```

---

## Task 2 — Add the `event_tx` field (`spf/src/game.rs`)

In `struct Game` (currently `game.rs:164`), add the sender as a non-serialized field
(mirrors the existing `#[serde(skip_serializing)]` fields `home`, `away`, `next_play`,
`fac_deck`):

```rust
#[derive(Serialize)]
pub struct Game {
    // ... existing fields unchanged ...

    #[serde(skip_serializing)]
    pub fac_deck: FacManager,

    /// Runtime plumbing: broadcasts domain events to transport adapters. Not game data,
    /// so it is skipped in serialization.
    #[serde(skip_serializing)]
    event_tx: broadcast::Sender<GameEvent>,
}
```

Notes:
- `Game` derives only `Serialize` (never `Deserialize` — verified), so a non-`Deserialize`
  field is fine; only the `skip_serializing` attribute is required.
- Keep the field **private** (`event_tx`, no `pub`): external access is via `subscribe()`.

---

## Task 3 — Build the channel + add `emit()` / `subscribe()` (`spf/src/game.rs`)

### 3a. Initialize in `create_game` (`game.rs:182`)

`create_game` returns `Self`. Build the sender before the struct literal and move it in:

```rust
pub fn create_game(home: Roster, away: Roster) -> Self {
    let start_type = PlayType::Kickoff;
    let (event_tx, _rx) = broadcast::channel(GAME_EVENT_CHANNEL_CAPACITY);

    let game = Self {
        home,
        away,
        state: GameState::start_state(),
        past_plays: vec![],
        next_play: Some(start_type.create_impl()),
        offlineup: None,
        deflineup: None,
        fac_deck: FacManager::new("cards/fac_cards.csv"),
        event_tx,
    };

    // No subscribers exist yet at creation (see stage doc D2); this is a deliberate no-op
    // today but preserves the "every mutation emits" invariant.
    game.emit(GameEvent::GameStarted { state: game.state });

    game
}
```

- The temporary `_rx` from `broadcast::channel` is dropped immediately; that is fine — the
  `Sender` stays alive and open as long as it (i.e. the `Game`) lives, and new receivers are
  minted on demand by `subscribe()`.
- `emit` takes `&self`, and `game.state` is `Copy`, so building the event after the struct
  exists is clean.

### 3b. Add the two methods in `impl Game`

```rust
/// Publish a domain event to all current subscribers. A send error means "no subscribers
/// right now", which is normal and intentionally ignored (architecture §4).
fn emit(&self, event: GameEvent) {
    let _ = self.event_tx.send(event);
}

/// Obtain a receiver for this game's event stream. Each transport adapter (e.g. the WS
/// handler in Stage 3) calls this to get its own independent receiver.
pub fn subscribe(&self) -> broadcast::Receiver<GameEvent> {
    self.event_tx.subscribe()
}
```

Placement: put both near the top of the `impl Game` block (e.g. right after `create_game`)
so the emitter surface is grouped and easy to find.

---

## Task 4 — Insert `emit()` calls at each mutation site (`spf/src/game.rs`)

Add one `emit(...)` per method, **after** the state change succeeds (so no event is emitted
on an error path). All these methods return `Result<_, String>`; emit only on the `Ok` path,
before returning `Ok`.

| Method (line, current) | Emit | Payload source |
|---|---|---|
| `create_game` (`:182`) | `GameEvent::GameStarted { state }` | `self.state` (see Task 3a) |
| `set_offensive_lineup_from_ids` (`:211`) | `GameEvent::OffensiveLineupSet { lineup }` | clone of the `id_lineup` arg (already cloned into `self.offlineup`) |
| `set_defensive_lineup_from_ids` (`:225`) | `GameEvent::DefensiveLineupSet { lineup }` | clone of the `id_lineup` arg |
| `set_next_play_type` (`:324`) | `GameEvent::NextPlayTypeSet { play_type }` | the `playtype` arg (it is `Copy`) |
| `run_current_play` (`:287`) | `GameEvent::PlayRun { play: Box::new(res.clone()) }` | the `PlayAndState` result, **boxed** |

### Detail — the two lineup setters

Each already does `self.offlineup = Some(id_lineup.clone());` (resp. `deflineup`). Emit the
same value. Example for offense (`set_offensive_lineup_from_ids`):

```rust
self.offlineup = Some(id_lineup.clone());
self.emit(GameEvent::OffensiveLineupSet { lineup: id_lineup.clone() });
Ok(())
```

> Note the **duplicate** `set_offense_lineup` / `set_defense_lineup` methods (`:256`, `:268`)
> are annotated `#[allow(dead_code)]` ("duplicate of the used …; kept pending removal"). Do
> **not** add emissions there — they have no callers. Emit only from the `*_from_ids`
> variants that the REST handlers actually call.

### Detail — `set_next_play_type` (`:324`)

Emit at the end of the `Ok` path, after the play impl is swapped in:

```rust
self.next_play = Some(playtype.create_impl());
if !same_type {
    self.offlineup = None;
    self.deflineup = None;
}
self.emit(GameEvent::NextPlayTypeSet { play_type: playtype });
Ok(())
```

### Detail — `run_current_play` (`:287`) and the D3 double-emit

`run_current_play` calls `self.set_next_play_type(...)` at `:300`, which now emits
`NextPlayTypeSet` on its own. Then emit `PlayRun` after it, mirroring the existing sequence:

```rust
self.past_plays.push(res.clone());
self.state = GameState { ..res.new_state };
self.set_next_play_type(self.state.get_next_move_default())?; // emits NextPlayTypeSet
self.emit(GameEvent::PlayRun { play: Box::new(res.clone()) }); // then PlayRun
Ok(res)
```

Net order for a single `POST /game/play`: `NextPlayTypeSet`, then `PlayRun`. This is
intended (D3). `res` is already cloned once for `past_plays`; the `Box::new(res.clone())`
adds one more clone for the event — acceptable and matches the architecture doc's example
(§4).

---

## Task 5 — Remove the Stage 1 `#[allow(dead_code)]` (`spf/src/game/events.rs`)

Now that every variant is constructed by a Task 4 emit, the unused-enum lint no longer
fires. Delete the temporary attribute added in Stage 1:

```rust
// DELETE this line from events.rs:
#[allow(dead_code)] // wired up in Stage 2; remove once `emit()`/`subscribe()` exist.
```

Leave the doc comment and derives intact. (If, and only if, clippy then flags a specific
still-unconstructed variant, revisit — but all five variants are emitted per Task 4, so it
should be clean.)

---

## Task 6 — Unit test (`spf/src/game.rs`, inline `#[cfg(test)] mod tests`)

Add an inline test module at the bottom of `game.rs` (the crate's second test module, after
`resulthandler.rs`). It must be **deterministic** and **dependency-free** (D4/D5).

### Building a `Game` without the full league

`create_game` needs two `Roster`s. **Verified constructor path (no card-data fixtures, test
always runs):**

- `TeamID` has public `String` fields (`players.rs:23`), so build one inline:
  `TeamID { name: "Home".into(), year: "1983".into() }`.
- `Roster::from_players(team_name, players)` is **public** (`players.rs:588`) and accepts an
  **empty** `Vec<Player>`. An empty roster is sufficient because the primary test only
  constructs a `Game` and calls `set_next_play_type`, which never touches the rosters.

```rust
use spf_core::players::{Player, Roster, TeamID};

fn empty_roster(name: &str) -> Roster {
    Roster::from_players(
        TeamID { name: name.into(), year: "1983".into() },
        Vec::<Player>::new(),
    )
}
// let mut game = Game::create_game(empty_roster("Home"), empty_roster("Away"));
```

> `Roster::create_roster(...)` is private; `from_players` is the public seam. `Roster` does
> **not** implement `Default`, which is why the empty-`Vec` `from_players` call is the chosen
> path rather than `Roster::default()`.

This keeps the roster construction dependency-free. **However — correction discovered during
implementation:** `create_game` also calls `FacManager::new("cards/fac_cards.csv")`, a
CWD-relative read. Cargo runs `spf`-crate tests with CWD = the crate dir, so that path is
unreachable and `FacManager::new` panics on `unwrap()`. Two consequences:

1. `create_game` was refactored to delegate to a private
   `create_game_with_fac_path(home, away, fac_path)` (public API unchanged) so the test can
   pass a reachable path.
2. The test uses `fac_path = "../cards/fac_cards.csv"` and **self-skips** when it is absent
   (`../../design/testing-strategy.md` §6), mirroring the `persist.rs` round-trip test. So the
   primary assertion runs whenever the FAC CSV is present (the normal checkout) and skips
   cleanly otherwise — it is not unconditionally always-on as an earlier draft implied.

The `run_current_play` → `PlayRun` contents assertion still needs real player rosters and the
FAC deck seam (**T3**); it remains out of scope here.

### Test body (primary — card-draw-independent)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use spf_core::players::{Player, Roster, TeamID};
    use std::path::Path;

    fn empty_roster(name: &str) -> Roster {
        Roster::from_players(
            TeamID { name: name.into(), year: "1983".into() },
            Vec::<Player>::new(),
        )
    }

    #[test]
    fn test_set_next_play_type_emits_event() {
        // Tests run with CWD = crate dir, so the workspace FAC deck is at `../cards/...`.
        // Self-skip (rather than panic in FacManager::new) when absent.
        let fac_path = "../cards/fac_cards.csv";
        if !Path::new(fac_path).exists() {
            eprintln!("skipping event-emission test: {} not present", fac_path);
            return;
        }

        // Arrange: a game from empty rosters + a reachable FAC path.
        let mut game =
            Game::create_game_with_fac_path(empty_roster("Home"), empty_roster("Away"), fac_path);

        // Subscribe *after* creation: the GameStarted emitted inside create_game had no
        // receiver and is already gone, so the channel is empty here.
        let mut rx = game.subscribe();

        // Act: a deterministic, non-card state change. From the start state
        // (last_status == Start) Kickoff is the only legal next type.
        game.set_next_play_type(PlayType::Kickoff)
            .expect("Kickoff is a legal next play type from the start state");

        // Assert: the event arrived and carries the right PlayType (PlayType: PartialEq).
        match rx.try_recv() {
            Ok(GameEvent::NextPlayTypeSet { play_type }) => {
                assert_eq!(play_type, PlayType::Kickoff);
            }
            other => panic!("expected NextPlayTypeSet, got {:?}", other),
        }
    }
}
```

Notes:
- Use `rx.try_recv()` (sync). `send` already delivered the event before the method returned.
- **Legality:** `set_next_play_type` validates against `get_next_move_types()`. From
  `start_state`, `last_status == Start` ⇒ only `Kickoff` is allowed. Use `Kickoff` (or first
  create a state where the target type is legal). Do **not** assert on a type the state
  rejects — that returns `Err` and emits nothing.
- **`Debug` for `panic!`:** `GameEvent` derives `Debug` (Stage 1) and `TryRecvError` is
  `Debug`, so the `{:?}` panic message compiles.

### Optional secondary assertion (only if a `Game` runs a play cheaply)

If (and only if) constructing a play-capable `Game` is trivial, a coarse
`matches!(rx.try_recv(), Ok(GameEvent::NextPlayTypeSet { .. }))` followed by
`matches!(rx.try_recv(), Ok(GameEvent::PlayRun { .. }))` documents the D3 ordering. Do **not**
inspect `PlayRun` contents (nondeterministic until T3). If running a play needs card data,
skip this and rely on the primary test.

---

## Files touched in Stage 2

| File | Change |
|---|---|
| `spf/src/game.rs` | `use tokio::sync::broadcast;` + `use self::events::GameEvent;`; `GAME_EVENT_CHANNEL_CAPACITY` const; `event_tx` field; `create_game` now delegates to a private `create_game_with_fac_path`; channel build there + `GameStarted` emit; `emit()` + `subscribe()` (`subscribe` carries `#[allow(dead_code)]` until Stage 3); 5 emit call-sites; inline `#[cfg(test)] mod tests` |
| `spf/src/game/events.rs` | Remove the temporary `#[allow(dead_code)]` |

Explicitly **not** touched: `webendpoint.rs`, `main.rs`, `Cargo.toml` (no new deps),
`engine.rs`, any handler.

---

## Verification checklist

1. `cargo build -p spf` — compiles; no new deps.
2. `cargo clippy -p spf` — clean. In particular, the `events.rs` unused-enum warning is gone
   (variants are now constructed) and no new warning is introduced by the field/methods.
   Watch for `clippy::large_enum_variant` on `GameEvent` — already mitigated by
   `Box<PlayAndState>` (Stage 1); do not un-box.
3. `cargo fmt -- --check` — formatting matches.
4. `cargo test -p spf` — the new test passes; the 13 existing `resulthandler` tests still
   pass (`cargo test --workspace` for the full 42).
5. Sanity: `game.subscribe()` is callable from outside the module (it is `pub`), readying
   Stage 3's WS handler.

---

## Risks & gotchas

- **Emitting on error paths.** Only emit after the mutation succeeds (on the `Ok` path). The
  lineup setters `?`-propagate the play-impl call first; place the `emit` after that and
  after the `self.offlineup = …` assignment.
- **Double-emit awareness (D3).** Reviewers may flag the `NextPlayTypeSet` + `PlayRun` pair
  from one `run_current_play`. It is intentional; the D3 note and an inline `//` comment at
  the call site should preempt confusion.
- **`GameStarted` no-op (D2).** Do not "optimize away" the `create_game` emission; it is
  intentional and harmless. The inline `//` comment explains why.
- **Capacity too small.** If a future stage adds burst-heavy events, revisit
  `GAME_EVENT_CHANNEL_CAPACITY`; `128` is sized for today's max burst of 2.
- **Test flakiness.** Avoid asserting on anything downstream of the FAC shuffle. The primary
  test uses `NextPlayTypeSet` precisely to stay deterministic without T3.

---

## Notes carried forward to Stage 3

- Stage 3 (`GET /game/ws`, snapshot-then-stream, both confirmed) consumes `subscribe()`:
  lock the game briefly, grab a `Receiver`, immediately push a snapshot frame (current
  `GameState`, e.g. as a synthesized `GameStarted`-shaped message), then forward each
  subsequent `GameEvent` as a JSON text frame; `Ping`→`Pong`; exit on `Close`/lag-closed.
- The deterministic `run_current_play` → `PlayRun` assertion (inspecting contents) lands with
  testing-plan **T3** (FAC deck seam) — not in Stage 2.
- Stage 4 registers `GameEvent` in the OpenAPI `ApiDoc` and documents the WS endpoint.
