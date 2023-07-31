use std::{cell::RefCell, sync::Mutex};

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};

use crate::game::{players::TeamList, Game, engine::{OffensiveLineup, DefensiveLineup}};

async fn set_offensive_lineup(
    appstate: web::Data<AppState>,
    lineup: web::Json<OffensiveLineup>,
) -> impl Responder {
    println!("{:?}", lineup); // Do something with the OffensiveLineup struct
    HttpResponse::Ok().body("Offensive lineup set.")
}

async fn get_offensive_lineup() -> impl Responder {
    // Generate the OffensiveLineup data (replace with your actual data source)
    // let offensive_lineup = generate_offensive_lineup();

    // // Convert the OffensiveLineup struct to JSON
    // let json_data = serde_json::to_string(&offensive_lineup)
    //     .expect("Error while serializing OffensiveLineup to JSON.");
    let json_data = r#"{
        "LE_split": null,
        "LE_tight": null,
        "RE_split": null,
        "RE_tight": null,
        "FL1": null,
        "FL2": null,
        "QB": null,
        "B1": null,
        "B2": null,
        "B3": null,
        "LT": null,
        "LG": null,
        "C": null,
        "RG": null,
        "RT": null
    }"#;

    // Set the Content-Type header to application/json
    HttpResponse::Ok()
        .content_type("application/json")
        .body(json_data)
}

async fn set_defensive_lineup(lineup: web::Json<DefensiveLineup>) -> impl Responder {
    println!("{:?}", lineup); // Do something with the DefensiveLineup struct
    HttpResponse::Ok().body("Defensive lineup set.")
}

async fn set_offensive_play(data: web::Json<(String, String)>) -> impl Responder {
    let (play1, play2) = data.into_inner();
    println!("Play 1: {}, Play 2: {}", play1, play2); // Do something with the plays
    HttpResponse::Ok().body("Offensive play set.")
}

async fn set_defensive_play(data: web::Json<(String, String)>) -> impl Responder {
    let (play1, play2) = data.into_inner();
    println!("Play 1: {}, Play 2: {}", play1, play2); // Do something with the plays
    HttpResponse::Ok().body("Defensive play set.")
}

async fn get_game_state(appstate: web::Data<AppState>) -> impl Responder {
    println!("Get State Called");

    let game = appstate.game.lock().unwrap();

    let state = game.state;

    // Convert the OffensiveLineup struct to JSON
    let json_data = serde_json::to_string(&state).expect("Error while serializing State to JSON.");

    // Set the Content-Type header to application/json
    HttpResponse::Ok()
        .content_type("application/json")
        .body(json_data)
}

async fn get_player(path: web::Path<String>, appstate: web::Data<AppState>) -> impl Responder {
    println!("Get Player Called: {:?}", path);
    let path_param = path.into_inner();
    println!("Get Player Called Inner: {:?}", path_param);

    let league = &appstate.league;

    let rec = league.get_player(&path_param);

    let json_data = match rec {
        Some(player) => player.get_json(),
        None => "{\"err\": \"No such player\"}".to_string(),
    };
    // println!("Rec**********");
    // println!("{}", json_data);

    // Respond with the player as JSON
    // let json_data = r#"{
    //     "LE_split": null,
    //     "LE_tight": null,
    //     "RE_split": null,
    //     "RE_tight": null,
    //     "FL1": null,
    //     "FL2": null,
    //     "QB": null,
    //     "B1": null,
    //     "B2": null,
    //     "B3": null,
    //     "LT": null,
    //     "LG": null,
    //     "C": null,
    //     "RG": null,
    //     "RT": null
    // }"#;
    HttpResponse::Ok()
        .content_type("application/json")
        .body(json_data)
}

struct AppState {
    game: Mutex<Game>,
    league: TeamList,
}

#[actix_web::main]
pub async fn runserver(game: Game, league: TeamList) -> std::io::Result<()> {
    let app_state = web::Data::new(AppState {
        game: Mutex::new(game),
        league,
    });

    // let game = RefCell::new(game);
    println!("Starting up server....");

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/setoffenselineup", web::post().to(set_offensive_lineup))
            .route("/getoffensivelineup", web::get().to(get_offensive_lineup))
            .route("/setdefenselineup", web::post().to(set_defensive_lineup))
            .route("/setoffensiveplay", web::post().to(set_offensive_play))
            .route("/setdefensiveplay", web::post().to(set_defensive_play))
            .route("/getstate", web::get().to(get_game_state))
            .route("/getplayer/{id}", web::get().to(get_player))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
