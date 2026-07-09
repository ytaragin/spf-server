use std::cmp::min;

use crate::game::{GamePlayStatus, GameState, GameTeams};

use super::{defs::GAMECONSTANTS, Down, PlayResult, ResultType};

pub fn calculate_play_result(old_state: &GameState, result: &PlayResult) -> GameState {
    let new_line = result.final_line;
    let (quarter, time_remaining) =
        advance_time(old_state.quarter, old_state.time_remaining, result.time);

    let interim_state = GameState {
        yard_line: new_line,
        time_remaining,
        quarter,
        play_counter: old_state.play_counter + 1,

        ..old_state.clone()
    };

    match result.result_type {
        ResultType::Regular => handle_regular_play(&interim_state, result),
        ResultType::TurnOver => handle_turnover(&interim_state),
    }
}

fn handle_regular_play(interim_state: &GameState, _result: &PlayResult) -> GameState {
    if interim_state.yard_line >= 100 {
        return handle_touchdown(interim_state);
    }

    if interim_state.yard_line < 0 {
        return handle_safety(interim_state);
    }

    if interim_state.yard_line >= interim_state.first_down_target {
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
        possession: interim_state.possession.other_team(),
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
    if interim_state.yard_line < 0 {
        let score_state = GameState {
            possession: interim_state.possession.other_team(),

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
        first_down_target: min(interim_state.yard_line + 10, 100),
        ..interim_state.clone()
    }
}

fn possession_change(interim_state: &GameState) -> GameState {
    let yard_line = 100 - interim_state.yard_line;

    GameState {
        down: Down::First,
        last_status: GamePlayStatus::PossessionChange,
        first_down_target: min(yard_line + 10, 100),
        possession: interim_state.possession.other_team(),
        yard_line,

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
    match interim_state.possession {
        GameTeams::Home => (interim_state.home_score + points, interim_state.away_score),
        GameTeams::Away => (interim_state.home_score, interim_state.away_score + points),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::engine::{CardResults, Yard};

    // ---- helpers -----------------------------------------------------------
    //
    // `GameState` / `PlayResult` are plain public-field structs, so tests build
    // them via struct-update from a known baseline rather than a builder type.
    // `GamePlayStatus` and `GameTeams` do not derive `PartialEq`, so status /
    // possession are compared by discriminant via the `is_status` / `is_possession`
    // helpers below.

    /// A mid-drive baseline: Away has the ball, 2nd & (target 60) at midfield,
    /// early in Q1 with plenty of time on the clock.
    fn base_state() -> GameState {
        GameState {
            down: Down::Second,
            yard_line: 50,
            first_down_target: 60,
            ..GameState::start_state()
        }
    }

    /// Build a `PlayResult` carrying only the fields the resolver reads
    /// (`result_type`, `final_line`, `time`); the rest are inert.
    fn play_result(result_type: ResultType, final_line: Yard, time: i32) -> PlayResult {
        PlayResult {
            result_type,
            result: 0,
            final_line,
            time,
            details: vec![],
            mechanic: vec![],
            extra: None,
            cards: CardResults::default(),
        }
    }

    fn is_status(state: &GameState, expected: GamePlayStatus) -> bool {
        std::mem::discriminant(&state.last_status) == std::mem::discriminant(&expected)
    }

    fn is_possession(state: &GameState, expected: GameTeams) -> bool {
        std::mem::discriminant(&state.possession) == std::mem::discriminant(&expected)
    }

    // ---- clock / bookkeeping ----------------------------------------------

    #[test]
    fn test_play_counter_increments_every_play() {
        let state = base_state();
        let before = state.play_counter;
        let new_state = calculate_play_result(&state, &play_result(ResultType::Regular, 52, 15));
        assert_eq!(
            new_state.play_counter,
            before + 1,
            "play_counter should advance by exactly one per play"
        );
    }

    #[test]
    fn test_regular_gain_short_of_marker_advances_down() {
        // 2nd down, gain to the 55 (target is 60) -> 3rd down, still ongoing.
        let state = base_state();
        let new_state = calculate_play_result(&state, &play_result(ResultType::Regular, 55, 15));

        assert_eq!(
            new_state.down,
            Down::Third,
            "down should advance 2nd -> 3rd"
        );
        assert_eq!(new_state.yard_line, 55, "ball spotted at the gained line");
        assert!(
            is_status(&new_state, GamePlayStatus::Ongoing),
            "a routine gain leaves the drive Ongoing"
        );
        assert!(
            is_possession(&new_state, GameTeams::Away),
            "possession is unchanged on a routine gain"
        );
    }

    #[test]
    fn test_regular_gain_reaching_marker_is_first_down() {
        // 2nd down, gain to the 62 (>= target 60) -> fresh 1st down.
        let state = base_state();
        let new_state = calculate_play_result(&state, &play_result(ResultType::Regular, 62, 15));

        assert_eq!(
            new_state.down,
            Down::First,
            "reaching the marker resets to 1st"
        );
        assert_eq!(
            new_state.first_down_target, 72,
            "new marker is 10 yards ahead of the new line (min with 100)"
        );
        assert_eq!(new_state.yard_line, 62);
        assert!(is_status(&new_state, GamePlayStatus::Ongoing));
        assert!(is_possession(&new_state, GameTeams::Away));
    }

    #[test]
    fn test_first_down_target_clamps_at_goal_line() {
        // Reach the marker deep in the red zone: new target is capped at 100.
        let state = GameState {
            down: Down::Second,
            yard_line: 50,
            first_down_target: 60,
            ..GameState::start_state()
        };
        let new_state = calculate_play_result(&state, &play_result(ResultType::Regular, 95, 15));

        assert_eq!(new_state.down, Down::First);
        assert_eq!(
            new_state.first_down_target, 100,
            "marker cannot extend past the goal line"
        );
    }

    #[test]
    fn test_fourth_down_short_turns_ball_over_on_downs() {
        // 4th down, short of the marker -> possession flips, field flips.
        let state = GameState {
            down: Down::Fourth,
            yard_line: 55,
            first_down_target: 60,
            ..GameState::start_state()
        };
        let new_state = calculate_play_result(&state, &play_result(ResultType::Regular, 57, 15));

        assert!(
            is_possession(&new_state, GameTeams::Home),
            "turnover on downs flips possession Away -> Home"
        );
        assert_eq!(
            new_state.yard_line, 43,
            "field flips: new line is 100 - old line (100 - 57)"
        );
        assert_eq!(
            new_state.down,
            Down::First,
            "new possession starts on 1st down"
        );
        assert_eq!(
            new_state.first_down_target, 53,
            "new marker 10 ahead (43 + 10)"
        );
        assert!(is_status(&new_state, GamePlayStatus::PossessionChange));
    }

    #[test]
    fn test_explicit_turnover_in_field_changes_possession() {
        // ResultType::TurnOver with the ball in the field of play (>= 0).
        let state = base_state();
        let new_state = calculate_play_result(&state, &play_result(ResultType::TurnOver, 58, 15));

        assert!(
            is_possession(&new_state, GameTeams::Home),
            "an interception/fumble in the field flips possession"
        );
        assert_eq!(new_state.yard_line, 42, "field flips (100 - 58)");
        assert_eq!(new_state.down, Down::First);
        assert!(is_status(&new_state, GamePlayStatus::PossessionChange));
    }

    #[test]
    fn test_offensive_touchdown_scores_for_team_in_possession() {
        // Away has the ball and reaches the end zone (>= 100).
        let state = base_state();
        let new_state = calculate_play_result(&state, &play_result(ResultType::Regular, 100, 15));

        assert!(is_status(&new_state, GamePlayStatus::Touchdown));
        assert_eq!(
            new_state.away_score, 6,
            "Away (in possession) gets the 6 points"
        );
        assert_eq!(new_state.home_score, 0, "Home does not score");
    }

    #[test]
    fn test_offensive_touchdown_scores_for_home_when_home_has_ball() {
        // Same TD, but Home is on offense -> the points land on Home.
        let state = GameState {
            possession: GameTeams::Home,
            ..base_state()
        };
        let new_state = calculate_play_result(&state, &play_result(ResultType::Regular, 104, 15));

        assert!(is_status(&new_state, GamePlayStatus::Touchdown));
        assert_eq!(new_state.home_score, 6);
        assert_eq!(new_state.away_score, 0);
    }

    #[test]
    fn test_turnover_into_end_zone_is_defensive_touchdown() {
        // TurnOver with final_line < 0 -> the *defense* (Home) scores.
        let state = base_state();
        let new_state = calculate_play_result(&state, &play_result(ResultType::TurnOver, -5, 15));

        assert!(is_status(&new_state, GamePlayStatus::Touchdown));
        assert_eq!(
            new_state.home_score, 6,
            "the team gaining possession (Home) is credited the defensive TD"
        );
        assert_eq!(new_state.away_score, 0);
    }

    #[test]
    fn test_regular_play_behind_own_goal_line_is_safety() {
        // Regular play, ball driven behind the goal line (< 0) -> safety,
        // points to the other team.
        let state = base_state();
        let new_state = calculate_play_result(&state, &play_result(ResultType::Regular, -2, 15));

        assert!(is_status(&new_state, GamePlayStatus::Safety));
        assert_eq!(
            new_state.home_score, 2,
            "safety awards 2 points to the defending team (Home)"
        );
        assert_eq!(new_state.away_score, 0);
    }

    // ---- clock rollover (advance_time via the public entry point) ----------

    #[test]
    fn test_clock_runs_down_within_quarter() {
        let state = base_state(); // Q1, 900s remaining
        let new_state = calculate_play_result(&state, &play_result(ResultType::Regular, 52, 40));

        assert_eq!(new_state.quarter, 1, "still in the same quarter");
        assert_eq!(new_state.time_remaining, 860, "900 - 40 seconds elapsed");
    }

    #[test]
    fn test_clock_expiring_advances_to_next_quarter() {
        // 30s left in Q1, a 45s play -> roll into Q2 with a full clock.
        let state = GameState {
            quarter: 1,
            time_remaining: 30,
            ..base_state()
        };
        let new_state = calculate_play_result(&state, &play_result(ResultType::Regular, 52, 45));

        assert_eq!(
            new_state.quarter, 2,
            "clock hitting zero advances the quarter"
        );
        assert_eq!(
            new_state.time_remaining, 900,
            "the new quarter starts with a full clock (sec_per_quarter)"
        );
    }

    #[test]
    fn test_clock_expiring_in_final_quarter_clamps_to_zero() {
        // Time expiring in Q4 stays in Q4 at 0:00 (no Q5).
        let state = GameState {
            quarter: 4,
            time_remaining: 20,
            ..base_state()
        };
        let new_state = calculate_play_result(&state, &play_result(ResultType::Regular, 52, 30));

        assert_eq!(new_state.quarter, 4, "no quarter beyond the last");
        assert_eq!(
            new_state.time_remaining, 0,
            "final-quarter clock clamps at 0"
        );
    }
}
