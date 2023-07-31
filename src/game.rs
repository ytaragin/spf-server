pub mod loader;
pub mod players;
pub mod stats;
pub mod engine;

use players::Player;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use self::{players::{Roster, TeamList}, engine::{Down, Play, OffensiveLineup, DefensiveLineup, OffensivePlay, DefensivePlay, PlayResult, Yard, Validatable}};



#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GameTeams {
    Home,
    Away,
}




#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GameState {
    pub quarter: i32,
    pub time_remaining: i32,
    pub possesion: GameTeams,
    pub down: Down,
    pub yardline: Yard,
    pub home_score: i32,
    pub away_score: i32,
}

impl GameState {
    pub fn start_state() -> Self {
        return Self {
            quarter: 1,
            time_remaining: 15 * 60,
            possesion: GameTeams::Home,
            down: Down::First,
            yardline: 50,
            home_score: 0,
            away_score: 0,
        };
    }
}

#[derive(Debug, Clone)]
pub struct PlayAndState {
    play: Play,
    result: PlayResult,
    new_state: GameState,

}

// #[derive(Debug, Clone)]
pub struct Game {
    pub home: Roster,
    pub away: Roster,
    pub state: GameState,
    pub past_plays: Vec<PlayAndState>,
    pub current_play: Play,

}

impl Game {
    pub fn create_game(home: Roster, away: Roster) -> Self {
        return Self {
            home,
            away,
            state: GameState::start_state(),
            past_plays: vec![],
            current_play: Play::new(),
        };
    }

    fn get_current_play(&mut self) -> &mut Play {

        return &mut self.current_play;
    }

    fn set_play_field<T, F>(&mut self, data: T, setter: F) -> Result<(), String>
    where
        F: Fn(&mut Play, T) -> (), T: Validatable
    {
        let play = self.get_current_play();

        if let Err(msg) = data.validate(play) {
            return Err(msg);
        }

        setter(play, data);
        return Ok(());
    }

    pub fn set_offensive_lineup(&mut self, lineup: OffensiveLineup) -> Result<(), String> {
        return self.set_play_field(lineup, |p: &mut Play, l| p.offense = Some(l));
    }

    pub fn set_defensive_lineup(&mut self, lineup: DefensiveLineup) -> Result<(), String> {
        return self.set_play_field(lineup, |p, l| p.defense = Some(l));
    }

    pub fn set_offense_play(&mut self, off_play: OffensivePlay) -> Result<(), String> {
        return self.set_play_field(off_play, |p, in_p| p.offense_play = Some(in_p));
    }

    pub fn set_defense_play(&mut self, def_play: DefensivePlay) -> Result<(), String> {
        return self.set_play_field(def_play, |p, in_p| p.defense_play = Some(in_p));
    }

    pub fn run_play(&mut self) -> Option<PlayAndState> {
        let play_result = self.current_play.run_play()?;

        let new_state = Game::gen_new_state(&self.state, &self.current_play, &play_result);
        let pands = PlayAndState {
            play: self.current_play.clone(),
            result: play_result,
            new_state
        };
        self.past_plays.push(pands.clone());

        self.state = new_state;
        self.current_play = Play::new();

        return Some(pands);
    }

    fn gen_new_state(curr_state: &GameState, play: &Play, result: &PlayResult) -> GameState {
        return GameState::start_state();
    }

}
