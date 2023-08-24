use std::sync::Mutex;

use actix_cors::Cors;
use actix_web::{http::header, web, App, HttpResponse, HttpServer, Responder};
use serde_json::json;

use crate::game::{
    lineup::{IDBasedDefensiveLineup, IDBasedOffensiveLineup},
    players::Serializable_Roster,
    Game,
};

async fn set_offensive_lineup(
    appstate: web::Data<AppState>,
    lineup: web::Json<IDBasedOffensiveLineup>,
) -> impl Responder {
    println!("{:?}", lineup); // Do something with the OffensiveLineup struct
    let mut game = appstate.game.lock().unwrap();
    let lineup_obj = lineup.into_inner();

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
    lineup: web::Json<IDBasedDefensiveLineup>,
) -> impl Responder {
    println!("{:?}", lineup); // Do something with the DefensiveLineup struct
    let mut game = appstate.game.lock().unwrap();
    let lineup_obj = lineup.into_inner();

    match game.set_defensive_lineup_from_ids(&lineup_obj) {
        Ok(_) => HttpResponse::Ok().body("Defensive lineup set."),
        Err(msg) => HttpResponse::BadRequest().body(msg),
    }
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
            .route("/setoffenselineup", web::post().to(set_offensive_lineup))
            .route("/getoffenselineup", web::get().to(get_offensive_lineup))
            .route("/getdefenselineup", web::get().to(get_defensive_lineup))
            .route("/setdefenselineup", web::post().to(set_defensive_lineup))
            // .route("/setoffensiveplay", web::post().to(set_offensive_play))
            // .route("/setdefensiveplay", web::post().to(set_defensive_play))
            .route("/getstate", web::get().to(get_game_state))
            .route("/getplayer/{id}", web::get().to(get_player))
            .route("/players/{team}", web::get().to(get_team_players))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
