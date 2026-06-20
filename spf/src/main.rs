mod game;
mod webendpoint;

extern crate spf_macros;

use game::{fac::read_csv_file, *};
// use lazy_static::lazy_static;

use crate::{
    game::{loader::*, players::TeamList},
    webendpoint::runserver,
};
// let static league: TeamList;

// lazy_static! {
//     static ref THE_GAME: Game = {
//         let league: TeamList = TeamList::create_teams("cards/SPFB1983");
//         Game::create_game(wash, dallas)
//     };
// }

fn main() {
    //     load_rbs("SPFB1983/83RB.txt");
    //     load_qbs("SPFB1983/83QB.txt");
    //     load_wrs("SPFB1983/83WR.txt");
    //     load_dbs("SPFB1983/83DB.txt");
    let league: TeamList = TeamList::create_teams("cards/SPFB1983");

    //     for v in league.teams.values() {
    //         v.print_team()
    //     }

    let _ = runserver(league);
}
