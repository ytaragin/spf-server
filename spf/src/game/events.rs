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
