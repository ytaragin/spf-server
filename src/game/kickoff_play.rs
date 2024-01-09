use serde_derive::Serialize;

use super::{players::{KRStats, KStats, Roster, Player}, engine::{PlayImpl, OffenseCall, DefenseCall, OffenseIDLineup, DefenseIDLineup, CardStreamer, PlayResult, PlayType, kickplay::KickPlayImpl}, GameState, Play};

#[derive(Debug, Clone, Default, Serialize)]
pub struct KickoffPlay {
    pub onside: Option<bool>,
    pub kr: Option<KRStats>,
    pub k: Option<KStats>,
}

impl KickoffPlay {
    pub fn new() -> Self {
        return Self {
            ..Default::default()
        };
    }
}

impl PlayImpl for KickoffPlay {
    fn validate(&self) -> Result<(), String> {
        let _ = self.onside.as_ref().ok_or("Offense not set");
        let _ = self.kr.as_ref().ok_or("Defense Lineup not set");
        let _ = self.k.as_ref().ok_or("Offense Lineup not set");
        Ok(())
    }

    fn set_offense_call(&mut self, call: OffenseCall) -> Result<(), String> {
        let c = call
            .as_kickoff_offense_call()
            .ok_or("Bad type".to_string())?;
        self.onside = Some(c.onside);
        Ok(())
    }

    fn set_defense_call(&mut self, call: DefenseCall) -> Result<(), String> {
        Ok(())
    }

    fn set_offense_lineup(
        &mut self,
        lineup: &OffenseIDLineup,
        roster: &Roster,
    ) -> Result<(), String> {
        let l = lineup
            .as_kickoff_id_offense_lineup()
            .ok_or("Bad type".to_string())?;

        self.k = Player::is_k(
            roster
                .get_player(&l.k)
                .ok_or(format!("Unknown player: {}", l.k))?
                .get_full_player(),
        );

        if self.k.is_none() {
            return Err("Player is not a K".to_string());
        }

        return Ok(());
    }

    fn set_defense_lineup(
        &mut self,
        lineup: &DefenseIDLineup,
        roster: &Roster,
    ) -> Result<(), String> {
        let l = lineup
            .as_kickoff_id_defense_lineup()
            .ok_or("Bad type".to_string())?;

        self.kr = Player::is_kr(
            roster
                .get_player(&l.kr)
                .ok_or(format!("Unknown player: {}", l.kr))?
                .get_full_player(),
        );

        if self.kr.is_none() {
            return Err("Player is not a KR".to_string());
        }

        return Ok(());
    }

    fn run_play<'a>(
        &'a self,
        game_state: &'a GameState,
        card_streamer: &'a mut CardStreamer<'a>,
    ) -> PlayResult {
        return KickPlayImpl::run_play(game_state, self, card_streamer);
    }

    fn get_play(&self) -> Play {
        return Play::Kickoff(self.clone());
    }

    fn get_type(&self) -> PlayType {
        PlayType::Kickoff
    }
}
