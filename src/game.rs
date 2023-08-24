pub mod engine;
pub mod lineup;
pub mod loader;
pub mod players;
pub mod stats;

use serde::{Deserialize, Serialize};

use self::{
    engine::{DefensivePlay, Down, OffensivePlayInfo, Play, PlayResult, Validatable, Yard, OffenseCall, DefenseCall},
    lineup::{DefensiveLineup, IDBasedDefensiveLineup, IDBasedOffensiveLineup, OffensiveLineup},
    players::Roster,
};

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

// // #[derive(Debug, Clone)]
// pub struct Game<'a> {
//     pub home:  &'a Roster,
//     pub away:  &'a Roster,
//     pub state: GameState,
//     pub past_plays: Vec<PlayAndState>,
//     pub current_play: Play,
// }

// impl<'a> Game<'a> {
//     pub fn create_game(home: &'a Roster, away: &'a Roster) -> Self {

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

    fn get_current_off_roster(&self) -> &Roster {
        match self.state.possesion {
            GameTeams::Away => &self.away,
            GameTeams::Home => &self.home,
        }
    }

    fn get_current_def_roster(&self) -> &Roster {
        match self.state.possesion {
            GameTeams::Away => &self.home,
            GameTeams::Home => &self.away,
        }
    }

    fn get_current_play(&mut self) -> &mut Play {
        return &mut self.current_play;
    }

    fn set_play_field<T, F>(&mut self, data: T, setter: F) -> Result<(), String>
    where
        F: Fn(&mut Play, T) -> (),
        T: Validatable,
    {
        let play = self.get_current_play();

        if let Err(msg) = data.validate(play) {
            return Err(msg);
        }

        setter(play, data);
        return Ok(());
    }

    pub fn set_offensive_lineup(&mut self, lineup: OffensiveLineup) -> Result<(), String> {
        let play = self.get_current_play();

        lineup.is_legal_lineup()?;
        play.offense = Some(lineup);
        return Ok(());

        // return self.set_play_field(lineup, |p: &mut Play, l| p.offense = Some(l));
    }
    pub fn set_offensive_lineup_from_ids(
        &mut self,
        id_lineup: &IDBasedOffensiveLineup,
    ) -> Result<(), String> {
        let lineup = OffensiveLineup::create_lineup(id_lineup, self.get_current_off_roster())?;

        return self.set_offensive_lineup(lineup);
    }

    pub fn set_defensive_lineup(&mut self, lineup: DefensiveLineup) -> Result<(), String> {
        let play = self.get_current_play();

        lineup.is_legal_lineup()?;
        play.defense = Some(lineup);
        return Ok(());
        // return self.set_play_field(lineup, |p, l| p.defense = Some(l));
    }

    pub fn set_defensive_lineup_from_ids(
        &mut self,
        id_lineup: &IDBasedDefensiveLineup,
    ) -> Result<(), String> {
        let lineup = DefensiveLineup::create_lineup(id_lineup, self.get_current_def_roster())?;
        return self.set_defensive_lineup(lineup);
    }

    pub fn set_offense_call(&mut self, off_call: OffenseCall) -> Result<(), String> {
        return self.set_play_field(off_call, |p, in_p| p.offense_call = Some(in_p));
    }

    pub fn set_defense_play(&mut self, def_call: DefenseCall) -> Result<(), String> {
        return self.set_play_field(def_call, |p, in_p| p.defense_call = Some(in_p));
    }

    pub fn get_offensive_lineup(&self) -> &Option<OffensiveLineup> {
        return &self.current_play.offense;
    }

    pub fn get_offensive_lineup_ids(&self) -> Option<IDBasedOffensiveLineup> {
        match &self.current_play.offense {
            None => None,
            Some(l) => Some(l.convert_to_id_lineup()),
        }
    }

    pub fn get_defensive_lineup_ids(&self) -> Option<IDBasedDefensiveLineup> {
        match &self.current_play.defense {
            None => None,
            Some(l) => Some(l.convert_to_id_lineup()),
        }
    }

    pub fn run_play(&mut self) -> Result<PlayAndState, String> {
        let play_result = self.current_play.run_play(&self.state)?;

        // let new_state = Game::gen_new_state(&self.state, &self.current_play, &play_result);
        // let pands = PlayAndState {
        //     play: self.current_play.clone(),
        //     result: play_result,
        //     new_state,
        // };
        self.past_plays.push(play_result.clone());

        self.state = play_result.new_state;
        self.current_play = Play::new();

        return Ok(play_result);
    }

    fn gen_new_state(curr_state: &GameState, play: &Play, result: &PlayResult) -> GameState {
        return GameState::start_state();
    }
}
