# Game Management: Construction, Environment & Layering

Durable reference for **how a `Game` is constructed and wired to the outside world**, and the
layering rules that keep that wiring clean. It covers the resource-loading boundary
(`GameEnvironment`), the construction API on `Game` (`create_game` / `build`), the
HTTP-to-domain boundary, and the ownership/sharing model that lets one environment back many
games.

**Out of scope (documented separately, later):** the *runtime* game lifecycle — starting a
game, setting lineups, running plays, clock/score/possession progression. This document is
only about **construction, the environment, and layering**.

Related: [`workspace-structure.md`](workspace-structure.md) (where these live),
[`testing-strategy.md`](testing-strategy.md) §5 (the FAC determinism seam),
[`ws-events-architecture.md`](ws-events-architecture.md) (`Game` as event emitter), and
[`../plans/tech-debt.md`](../plans/tech-debt.md) §1 (the CWD-path issue this design resolves).

---

## 1. Layering & responsibilities

Three layers, each with a single job:

| Layer | Responsibility | Must **not** |
|---|---|---|
| **`main`** | Load all external resources once at startup (`GameEnvironment::load`) and hand the result to the server. Handle load failure by exiting with an error. | Contain game logic. |
| **`webendpoint` (HTTP)** | A **slim REST wrapper**: deserialize the request, call one domain function, map its `Result` to an HTTP status/body. | Load resources, look teams/players up, validate game rules, or hold business logic. |
| **Domain (`Game`, `GameEnvironment`, engine)** | All game construction and rules: resolve teams, build the game, run plays, return typed domain errors. | Know about HTTP, actix, or file paths (paths live only in `main` + `GameEnvironment::load`). |

### The slim-endpoint rule

HTTP handlers exist only to translate between the wire and the domain. Concretely, a handler
should look like: parse → call domain → match result → respond. For example, `start_game`:

```rust
let game = match Game::create_game(&appstate.env, &req.home, &req.away) {
    Ok(g) => g,
    Err(CreateGameError::UnknownTeam(team)) =>
        return HttpResponse::NotFound().body(format!("Unknown team: {}", team.to_string())),
};
```

The team-membership check is **not** in the handler — it lives in `create_game`, which returns
`CreateGameError`. The handler's only job is to map that error to `404`. When adding a new
endpoint, push every decision that isn't "which HTTP status?" down into the domain.

---

## 2. `GameEnvironment`

`GameEnvironment` (`spf/src/game/environment.rs`) is the shared, read-only, long-lived bundle
of external data every game needs:

```rust
pub struct GameEnvironment {
    league: TeamList,     // read-only during play; shared across all games
    fac_deck: FacManager, // a template; each game gets its own clone
}
```

- **It is the single disk-loading site for game data.** `GameEnvironment::load(data_dir,
  fac_path)` loads the league (`persist::load_league`) and the FAC deck
  (`FacManager::from_csv`) and returns `Result<Self, String>`. Because loading returns a
  `Result`, a missing/mislocated file surfaces as a handled startup error instead of a panic
  (this closes [`tech-debt.md`](../plans/tech-debt.md) §1). No other module reads these
  resources from disk.
- **Created once, in `main`**, then stored in the server's `AppState` and borrowed for the
  life of the process.
- **Accessors** keep the fields private and encode intent: `league()` (read-only league
  access for endpoints), `roster(id)` (resolve a team), `new_deck()` (a fresh per-game deck).

---

## 3. Construction API on `Game`

Two entry points, layered:

```rust
// Public: resolve team ids against the environment, then build. Borrows the environment.
pub fn create_game(
    env: &GameEnvironment,
    home: &TeamID,
    away: &TeamID,
) -> Result<Game, CreateGameError>;

// Private: pure dependency-injected constructor. No disk, no lookups.
fn build(home: Roster, away: Roster, fac_deck: FacManager) -> Game;
```

- **`create_game`** owns the construction *logic*: it resolves each `TeamID` against the
  league and returns `CreateGameError::UnknownTeam(id)` when a team is absent — the check that
  used to live in the HTTP handler. On success it clones the two rosters and asks the
  environment for a fresh deck, then delegates to `build`.
- **`build`** is a pure DI constructor: it takes already-resolved rosters and an owned
  `FacManager`. It touches no disk and no globals, which makes it the seam tests use to inject
  a deterministic deck (see §5 below and `testing-strategy.md` §5).

### `CreateGameError`

A dedicated domain error so the HTTP layer stays a thin mapper:

```rust
pub enum CreateGameError {
    UnknownTeam(TeamID), // carries the offending id so the handler can format a 404
}
```

Add variants here (not `String`s) as construction grows more failure modes; each maps to one
HTTP status at the boundary.

---

## 4. Ownership & sharing model (multi-game ready)

The design assumes a future where **one `GameEnvironment` backs many concurrent games**. That
drives every borrow/clone decision:

| Data | Mutated while a game runs? | Sharing strategy |
|---|---|---|
| `GameEnvironment` | No | **Borrowed, never consumed.** `create_game` takes `&GameEnvironment`, so one environment serves N games. |
| `league` (`TeamList`) | No (read-only lookups) | **Shared, never cloned** into a game. A game only needs the two resolved rosters, not the league. |
| `fac_deck` (`FacManager`) | **Yes** — the deck is consumed as cards are drawn | **Per-game clone.** `new_deck()` clones the template so each game owns an independent deck. Two games must not share one deck. |
| rosters (`Roster`) | Per-game state | Cloned into the game (unavoidable; the game owns them). |

Because `create_game` **borrows** the environment, the signature is already forward-compatible
with sharing it behind `Arc<GameEnvironment>` or actix's `web::Data<GameEnvironment>` (both
deref to `&GameEnvironment`). **No construction-API change is needed to go multi-game.** The
only future change is on the storage side — `AppState.game` becomes a keyed collection
(e.g. `Mutex<HashMap<GameId, Game>>`) instead of `Mutex<Option<Game>>` — which is out of scope
here.

---

## 5. FAC determinism seam (pointer)

`FacManager` is the sole source of engine nondeterminism (the deck shuffle). It exposes two
constructors:

- `from_csv(path) -> Result<..>` — the production, shuffling deck (used by
  `GameEnvironment::load`).
- `from_cards(Vec<FacCard>)` — an in-memory, **ordered, non-shuffling** deck.

`from_cards` combined with `Game::build` lets a test construct a fully deterministic game with
a known draw order. The deep testing rationale lives in
[`testing-strategy.md`](testing-strategy.md) §5; this section is only a pointer so the
construction story is complete.
