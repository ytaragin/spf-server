use std::{str::FromStr, sync::Mutex};

use actix_cors::Cors;
use actix_web::{http::header, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use serde_json::json;

use crate::game::{
    engine::{DefenseCall, DefenseIDLineup, OffenseCall, OffenseIDLineup, PlayType},
    players::Serializable_Roster,
    Game, PlayAndState,
};

#[derive(Deserialize)]
struct PlayQueryParams {
    #[serde(default)]
    result: bool,
    count: Option<usize>,
}

fn serialize_plays(plays: &[PlayAndState], result_only: bool) -> Result<String, serde_json::Error> {
    if result_only {
        // Return only PlayResult and GameState for each play
        let partial_response: Vec<_> = plays
            .iter()
            .map(|play| {
                json!({
                    "result": play.result,
                    "new_state": play.new_state
                })
            })
            .collect();
        serde_json::to_string(&partial_response)
    } else {
        // Return full PlayAndState objects
        serde_json::to_string(plays)
    }
}

async fn set_offensive_lineup(
    appstate: web::Data<AppState>,
    lineup: web::Json<OffenseIDLineup>,
) -> impl Responder {
    println!("{:?}", lineup); // Do something with the OffensiveLineup struct
    let mut game = appstate.game.lock().unwrap();
    let lineup_obj = lineup.into_inner();

    println!("{:?}", lineup_obj); // Do something with the OffensiveLineup struct

    match game.set_offensive_lineup_from_ids(&lineup_obj) {
        Ok(_) => HttpResponse::Ok().body("Offensive lineup set."),
        Err(msg) => HttpResponse::BadRequest().body(msg),
    }
}

async fn get_offensive_lineup(appstate: web::Data<AppState>) -> impl Responder {
    println!("get_offensive_lineup called");

    let game = appstate.game.lock().unwrap();
    let lineup = game.get_offensive_lineup_ids();
    let res = serde_json::to_string(&lineup);

    return match res {
        Err(msg) => HttpResponse::InternalServerError().body(msg.to_string()),
        Ok(v) => HttpResponse::Ok().content_type("application/json").body(v),
    };
}

async fn get_defensive_lineup(appstate: web::Data<AppState>) -> impl Responder {
    println!("get_defensive_lineup called");
    let game = appstate.game.lock().unwrap();
    let lineup = game.get_defensive_lineup_ids();
    let res = serde_json::to_string(&lineup);

    return match res {
        Err(msg) => HttpResponse::InternalServerError().body(msg.to_string()),
        Ok(v) => HttpResponse::Ok().content_type("application/json").body(v),
    };
}

async fn set_defensive_lineup(
    appstate: web::Data<AppState>,
    lineup: web::Json<DefenseIDLineup>,
) -> impl Responder {
    println!("{:?}", lineup); // Do something with the DefensiveLineup struct
    let mut game = appstate.game.lock().unwrap();
    let lineup_obj = lineup.into_inner();

    match game.set_defensive_lineup_from_ids(&lineup_obj) {
        Ok(_) => HttpResponse::Ok().body("Defensive lineup set."),
        Err(msg) => HttpResponse::BadRequest().body(msg),
    }
}

async fn set_offense_call(
    appstate: web::Data<AppState>,
    data: web::Json<OffenseCall>,
) -> impl Responder {
    println!("data {:?}", data);

    let call = data.into_inner();
    println!("Offense Play:  {:?}", call); // Do something with the plays
    let mut game = appstate.game.lock().unwrap();

    match game.set_offense_call(call) {
        Ok(_) => HttpResponse::Ok().body("Offense play set."),
        Err(msg) => HttpResponse::BadRequest().body(msg),
    }
}

async fn set_defense_call(
    appstate: web::Data<AppState>,
    data: web::Json<DefenseCall>,
) -> impl Responder {
    let call = data.into_inner();
    println!("Defense Play:  {:?}", call); // Do something with the plays
    let mut game = appstate.game.lock().unwrap();

    match game.set_defense_call(call) {
        Ok(_) => HttpResponse::Ok().body("Defense play set."),
        Err(msg) => HttpResponse::BadRequest().body(msg),
    }
}

async fn run_play(appstate: web::Data<AppState>) -> impl Responder {
    println!("Running Play..."); // Do something with the plays
    let mut game = appstate.game.lock().unwrap();

    match game.run_current_play() {
        Ok(res) => {
            let json_data =
                serde_json::to_string(&res.result).expect("Error while serializing State to JSON.");
            HttpResponse::Ok()
                .content_type("application/json")
                .body(json_data)
        }
        Err(msg) => HttpResponse::BadRequest().body(msg),
    }
}

async fn get_team_players(
    team: web::Path<String>,
    appstate: web::Data<AppState>,
) -> impl Responder {
    let game = appstate.game.lock().unwrap();

    let team_path = team.into_inner();
    let team_rost = match team_path.as_str() {
        "home" => &game.home,
        "away" => &game.away,
        _ => return HttpResponse::NotFound().finish(),
    };

    let srost = Serializable_Roster::from_roster(team_rost);
    let json_data = serde_json::to_string(&srost).expect("Error while serializing State to JSON.");

    HttpResponse::Ok()
        .content_type("application/json")
        .body(json_data)
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

async fn get_next_play_types(appstate: web::Data<AppState>) -> impl Responder {
    println!("Get Next Plays Called");

    let game = appstate.game.lock().unwrap();

    let next_types = game.allowed_play_types();

    // Convert the OffensiveLineup struct to JSON
    let json_data = serde_json::to_string(&next_types)
        .expect("Error while serializing Alllowed Plays to JSON.");

    // Set the Content-Type header to application/json
    HttpResponse::Ok()
        .content_type("application/json")
        .body(json_data)
}

async fn set_next_play_type(appstate: web::Data<AppState>, data: String) -> impl Responder {
    println!("Set Next Play Called");

    println!("Play Type is {}", data);
    let v = PlayType::from_str(&data);
    if v.is_err() {
        return HttpResponse::BadRequest().body("Unknown Type");
    }

    let mut game = appstate.game.lock().unwrap();
    let res = game.set_next_play_type(v.unwrap());
    match res {
        Ok(_) => HttpResponse::Ok()
            .content_type("application/json")
            .body("Set"),
        Err(msg) => HttpResponse::BadRequest().body(msg),
    }
}

async fn save_game(appstate: web::Data<AppState>, data: String) -> impl Responder {
    println!("Save Game Called");

    println!("File is {}", data);

    let game = appstate.game.lock().unwrap();
    let res = game.serialize_struct(data);
    match res {
        Ok(_) => HttpResponse::Ok()
            .content_type("application/json")
            .body("Set"),
        Err(msg) => HttpResponse::BadRequest().body(msg.to_string()),
    }
}

async fn get_player(path: web::Path<String>, appstate: web::Data<AppState>) -> impl Responder {
    println!("Get Player Called: {:?}", path);
    let path_param = path.into_inner();
    println!("Get Player Called Inner: {:?}", path_param);

    let game = appstate.game.lock().unwrap();

    let mut rec = game.home.get_player(&path_param);
    if rec.is_none() {
        rec = game.away.get_player(&path_param);
    }

    // let league = &appstate.league;

    // let rec = league.get_player(&path_param);

    let json_data = match rec {
        Some(player) => player.get_json(),
        None => json!({"err": "No such player"}),
    };

    let json_str =
        serde_json::to_string(&json_data).unwrap_or("{\"err\": \"No such player\"}".to_string());

    HttpResponse::Ok()
        .content_type("application/json")
        .body(json_str)
}

async fn get_all_plays(
    appstate: web::Data<AppState>,
    query: web::Query<PlayQueryParams>,
) -> impl Responder {
    let game = appstate.game.lock().unwrap();
    let all_plays = game.get_all_plays();

    // Apply count filter if specified
    let plays_to_return = if let Some(count) = query.count {
        if all_plays.is_empty() {
            all_plays
        } else {
            let start_index = if count >= all_plays.len() {
                0
            } else {
                all_plays.len() - count
            };
            &all_plays[start_index..]
        }
    } else {
        all_plays
    };

    let json_data = serialize_plays(plays_to_return, query.result)
        .expect("Error while serializing plays to JSON.");

    HttpResponse::Ok()
        .content_type("application/json")
        .body(json_data)
}

struct AppState {
    // game: Arc<Mutex<Game<'a>>>,
    game: Mutex<Game>,
}

#[actix_web::main]
pub async fn runserver(game: Game) -> std::io::Result<()> {
    let app_state = web::Data::new(AppState {
        game: Mutex::new(game),
    });

    // let game = RefCell::new(game);
    println!("Starting up server....");

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:5173") // <- your vue app origin
                    .allowed_methods(vec!["GET", "POST"]) // <- allow GET and POST requests
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .max_age(3600)
                    .allow_any_origin(), // You can use `allow_origin` to specify a specific origin
                                         // .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                                         // .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                                         // .max_age(3600), // Optional: set max age for preflight requests
            )
            .route("/offense/lineup", web::post().to(set_offensive_lineup))
            .route("/offense/lineup", web::get().to(get_offensive_lineup))
            .route("/offense/call", web::post().to(set_offense_call))
            .route("/defense/lineup", web::get().to(get_defensive_lineup))
            .route("/defense/lineup", web::post().to(set_defensive_lineup))
            .route("/defense/call", web::post().to(set_defense_call))
            .route("/game/play", web::post().to(run_play))
            .route("/game/plays", web::get().to(get_all_plays))
            .route("/game/save", web::post().to(save_game))
            .route("/game/state", web::get().to(get_game_state))
            .route("/game/nexttype", web::get().to(get_next_play_types))
            .route("/game/nexttype", web::post().to(set_next_play_type))
            .route("/getplayer/{id}", web::get().to(get_player))
            .route("/players/{team}", web::get().to(get_team_players))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
