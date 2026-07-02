use serde_derive::{Deserialize, Serialize};
use strum_macros::EnumString;

/// Trait describing how a two-outcome ranged stat shifts its boundary.
///
/// Extracted from the play engine so the shared data model (`stats`, `players`)
/// can depend on it without pulling in the full play-execution machinery.
pub trait Shiftable<T> {
    fn get_first() -> T;
    fn get_second() -> T;
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize, EnumString)]
pub enum PassResult {
    #[strum(serialize = "Com")]
    Complete,
    #[strum(serialize = "Inc")]
    Incomplete,
    #[strum(serialize = "Int")]
    Interception,
}

impl Shiftable<PassResult> for PassResult {
    fn get_first() -> PassResult {
        PassResult::Complete
    }

    fn get_second() -> PassResult {
        PassResult::Incomplete
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize, EnumString)]
pub enum PassRushResult {
    #[strum(serialize = "Sack")]
    Sack,
    #[strum(serialize = "Runs")]
    Runs,
    #[strum(serialize = "Com")]
    Complete,
    #[strum(serialize = "Inc")]
    Incomplete,
}

impl Shiftable<PassRushResult> for PassRushResult {
    fn get_first() -> PassRushResult {
        PassRushResult::Sack
    }

    fn get_second() -> PassRushResult {
        PassRushResult::Runs
    }
}
