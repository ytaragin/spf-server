use std::cmp::{max, min};

use crate::game::{GamePlayStatus, GameState, GameTeams};

use super::{Down, Play, PlayResult, ResultType, GAMECONSTANTS};

pub fn calculate_play_result(old_state: &GameState, result: &PlayResult) -> GameState {
    let new_line = old_state.yardline + result.result;
    let (quarter, time_remaining) =
        advance_time(old_state.quarter, old_state.time_remaining, result.time);

    let interim_state = GameState {
        yardline: new_line,
        time_remaining,
        quarter,

        ..old_state.clone()
    };

    match result.result_type {
        ResultType::Regular => handle_regular_play(&interim_state, result),
        ResultType::TurnOver => handle_turnover(&interim_state),
    }
}

fn handle_regular_play(interim_state: &GameState, result: &PlayResult) -> GameState {
    if interim_state.yardline >= 100 {
        return handle_touchdown(interim_state);
    }

    if interim_state.yardline < 0 {
        return handle_safety(interim_state);
    }

    if interim_state.yardline >= interim_state.first_down_target {
        return first_down(&interim_state);
    }

    if interim_state.down == Down::Fourth {
        return possession_change(&interim_state);
    }

    GameState {
        down: interim_state.down.next_down(),
        last_status: GamePlayStatus::Ongoing,
        ..interim_state.clone()
    }
}

fn handle_safety(interim_state: &GameState) -> GameState {
    let score_state = GameState {
        possesion: interim_state.possesion.other_team(),
        ..interim_state.clone()
    };

    let (home_score, away_score) = add_points(&score_state, GAMECONSTANTS.points_for_safety);
    GameState {
        last_status: GamePlayStatus::Safety,
        home_score,
        away_score,
        ..interim_state.clone()
    }
}

fn handle_turnover(interim_state: &GameState) -> GameState {
    if interim_state.yardline < 0 {
        let score_state = GameState {
            possesion: interim_state.possesion.other_team(),

            ..interim_state.clone()
        };
        return handle_touchdown(&score_state);
    }

    possession_change(interim_state)
}

fn handle_touchdown(interim_state: &GameState) -> GameState {
    let (home_score, away_score) = add_points(interim_state, GAMECONSTANTS.points_for_td);

    GameState {
        last_status: GamePlayStatus::Touchdown,
        home_score,
        away_score,
        ..interim_state.clone()
    }
}

fn first_down(interim_state: &GameState) -> GameState {
    GameState {
        down: Down::First,
        last_status: GamePlayStatus::Ongoing,
        first_down_target: min(interim_state.yardline + 10, 100),
        ..interim_state.clone()
    }
}

fn possession_change(interim_state: &GameState) -> GameState {
    GameState {
        down: Down::First,
        last_status: GamePlayStatus::PossesionChange,
        first_down_target: min(interim_state.yardline + 10, 100),
        possesion: interim_state.possesion.other_team(),
        yardline: 100 - interim_state.yardline,

        ..interim_state.clone()
    }
}

fn advance_time(curr_quarter: i32, curr_time: i32, play_time: i32) -> (i32, i32) {
    let new_remaining = curr_time - play_time;
    if new_remaining <= 0 {
        let new_qtr = curr_quarter + 1;
        if new_qtr > GAMECONSTANTS.quarters {
            return (GAMECONSTANTS.quarters, 0);
        }
        return (new_qtr, GAMECONSTANTS.sec_per_quarter);
    }
    return (curr_quarter, new_remaining);
}

fn add_points(interim_state: &GameState, points: i32) -> (i32, i32) {
    match interim_state.possesion {
        GameTeams::Home => (interim_state.home_score + points, interim_state.away_score),
        GameTeams::Away => (interim_state.home_score, interim_state.away_score + points),
    }
}
