mod game;
mod webendpoint;

use game::*;
use players::{Player, Position};

use crate::{
    game::{
        loader::*,
        players::{TeamID, TeamList},
    },
    webendpoint::runserver,
};


fn main() {
    println!("Hello, world!");

    //     load_rbs("SPFB1983/83RB.txt");
    //     load_qbs("SPFB1983/83QB.txt");
    //     load_wrs("SPFB1983/83WR.txt");
    //     load_dbs("SPFB1983/83DB.txt");
    let league = TeamList::create_teams();
    let wash = league
        .teams
        .get(&TeamID {
            name: "Washington".to_string(),
            year: "1983".to_string(),
        })
        .unwrap();
    wash.print_team();

    let dallas = league
        .teams
        .get(&TeamID {
            name: "Dallas".to_string(),
            year: "1983".to_string(),
        })
        .unwrap();
    wash.print_team();

    let g = Game::create_game(wash, dallas);

    println!("{:?}", g);

    //     for v in league.teams.values() {
    //         v.print_team()
    //     }

    runserver(g, league);
}
