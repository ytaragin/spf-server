use std::{str::FromStr, sync::Mutex};

use actix_cors::Cors;
use actix_web::{
    get, http::header, post, rt, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use actix_ws::Message;
use futures_util::StreamExt;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::broadcast::error::RecvError;
use utoipa::{IntoParams, OpenApi, ToSchema};
use utoipa_actix_web::{scope, AppExt};
use utoipa_swagger_ui::SwaggerUi;

use crate::game::{
    engine::{DefenseCall, DefenseIDLineup, OffenseCall, OffenseIDLineup, PlayResult, PlayType},
    environment::GameEnvironment,
    events::GameEvent,
    players::{Serializable_Roster, TeamID},
    CreateGameError, Game, GameState, PlayAndState, PlayTypeInfo,
};

#[derive(Deserialize, ToSchema)]
struct StartGameRequest {
    home: TeamID,
    away: TeamID,
}

/// Locks the shared game state and binds `$game` to a `&mut Game`.
/// Early-returns 409 Conflict when no game is in progress.
macro_rules! lock_game {
    ($appstate:expr, $game:ident) => {
        let mut guard = $appstate.game.lock().unwrap();
        let $game = match guard.as_mut() {
            Some(g) => g,
            None => return HttpResponse::Conflict().body("No game in progress"),
        };
    };
}

#[derive(Deserialize, IntoParams)]
struct PlayQueryParams {
    /// When `true`, each entry contains only `{result, new_state}` instead of the full play.
    #[serde(default)]
    result: bool,
    /// Limit the response to the last `count` plays.
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

#[utoipa::path(
    tag = "offense",
    request_body = OffenseIDLineup,
    responses(
        (status = 200, description = "Offensive lineup set"),
        (status = 400, description = "Invalid lineup"),
        (status = 409, description = "No game in progress"),
    )
)]
#[post("/lineup")]
async fn set_offensive_lineup(
    appstate: web::Data<AppState>,
    lineup: web::Json<OffenseIDLineup>,
) -> impl Responder {
    println!("{:?}", lineup); // Do something with the OffensiveLineup struct
    lock_game!(appstate, game);
    let lineup_obj = lineup.into_inner();

    println!("{:?}", lineup_obj); // Do something with the OffensiveLineup struct

    match game.set_offensive_lineup_from_ids(&lineup_obj) {
        Ok(_) => HttpResponse::Ok().body("Offensive lineup set."),
        Err(msg) => HttpResponse::BadRequest().body(msg),
    }
}

#[utoipa::path(
    tag = "offense",
    responses(
        (status = 200, description = "Current offensive lineup", body = OffenseIDLineup),
        (status = 409, description = "No game in progress"),
        (status = 500, description = "Serialization error"),
    )
)]
#[get("/lineup")]
async fn get_offensive_lineup(appstate: web::Data<AppState>) -> impl Responder {
    println!("get_offensive_lineup called");

    lock_game!(appstate, game);
    let lineup = game.get_offensive_lineup_ids();
    let res = serde_json::to_string(&lineup);

    return match res {
        Err(msg) => HttpResponse::InternalServerError().body(msg.to_string()),
        Ok(v) => HttpResponse::Ok().content_type("application/json").body(v),
    };
}

#[utoipa::path(
    tag = "defense",
    responses(
        (status = 200, description = "Current defensive lineup", body = DefenseIDLineup),
        (status = 409, description = "No game in progress"),
        (status = 500, description = "Serialization error"),
    )
)]
#[get("/lineup")]
async fn get_defensive_lineup(appstate: web::Data<AppState>) -> impl Responder {
    println!("get_defensive_lineup called");
    lock_game!(appstate, game);
    let lineup = game.get_defensive_lineup_ids();
    let res = serde_json::to_string(&lineup);

    return match res {
        Err(msg) => HttpResponse::InternalServerError().body(msg.to_string()),
        Ok(v) => HttpResponse::Ok().content_type("application/json").body(v),
    };
}

#[utoipa::path(
    tag = "defense",
    request_body = DefenseIDLineup,
    responses(
        (status = 200, description = "Defensive lineup set"),
        (status = 400, description = "Invalid lineup"),
        (status = 409, description = "No game in progress"),
    )
)]
#[post("/lineup")]
async fn set_defensive_lineup(
    appstate: web::Data<AppState>,
    lineup: web::Json<DefenseIDLineup>,
) -> impl Responder {
    println!("{:?}", lineup); // Do something with the DefensiveLineup struct
    lock_game!(appstate, game);
    let lineup_obj = lineup.into_inner();

    match game.set_defensive_lineup_from_ids(&lineup_obj) {
        Ok(_) => HttpResponse::Ok().body("Defensive lineup set."),
        Err(msg) => HttpResponse::BadRequest().body(msg),
    }
}

#[utoipa::path(
    tag = "offense",
    request_body = OffenseCall,
    responses(
        (status = 200, description = "Offense play set"),
        (status = 400, description = "Invalid call"),
        (status = 409, description = "No game in progress"),
    )
)]
#[post("/call")]
async fn set_offense_call(
    appstate: web::Data<AppState>,
    data: web::Json<OffenseCall>,
) -> impl Responder {
    println!("data {:?}", data);

    let call = data.into_inner();
    println!("Offense Play:  {:?}", call); // Do something with the plays
    lock_game!(appstate, game);

    match game.set_offense_call(call) {
        Ok(_) => HttpResponse::Ok().body("Offense play set."),
        Err(msg) => HttpResponse::BadRequest().body(msg),
    }
}

#[utoipa::path(
    tag = "defense",
    request_body = DefenseCall,
    responses(
        (status = 200, description = "Defense play set"),
        (status = 400, description = "Invalid call"),
        (status = 409, description = "No game in progress"),
    )
)]
#[post("/call")]
async fn set_defense_call(
    appstate: web::Data<AppState>,
    data: web::Json<DefenseCall>,
) -> impl Responder {
    let call = data.into_inner();
    println!("Defense Play:  {:?}", call); // Do something with the plays
    lock_game!(appstate, game);

    match game.set_defense_call(call) {
        Ok(_) => HttpResponse::Ok().body("Defense play set."),
        Err(msg) => HttpResponse::BadRequest().body(msg),
    }
}

#[utoipa::path(
    tag = "game",
    responses(
        (status = 200, description = "Result of the executed play", body = PlayResult),
        (status = 400, description = "Play could not be run"),
        (status = 409, description = "No game in progress"),
    )
)]
#[post("/play")]
async fn run_play(appstate: web::Data<AppState>) -> impl Responder {
    println!("Running Play..."); // Do something with the plays
    lock_game!(appstate, game);

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

#[utoipa::path(
    tag = "players",
    params(("team" = String, Path, description = "Team selector: home | away")),
    responses(
        (status = 200, description = "Roster for the selected team", body = Serializable_Roster),
        (status = 404, description = "Unknown team selector"),
        (status = 409, description = "No game in progress"),
    )
)]
#[get("/players/{team}")]
async fn get_team_players(
    team: web::Path<String>,
    appstate: web::Data<AppState>,
) -> impl Responder {
    lock_game!(appstate, game);

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

#[utoipa::path(
    tag = "game",
    responses(
        (status = 200, description = "Current game state", body = GameState),
        (status = 409, description = "No game in progress"),
    )
)]
#[get("/state")]
async fn get_game_state(appstate: web::Data<AppState>) -> impl Responder {
    println!("Get State Called");

    lock_game!(appstate, game);

    let state = game.state;

    // Convert the OffensiveLineup struct to JSON
    let json_data = serde_json::to_string(&state).expect("Error while serializing State to JSON.");

    // Set the Content-Type header to application/json
    HttpResponse::Ok()
        .content_type("application/json")
        .body(json_data)
}

#[utoipa::path(
    tag = "game",
    responses(
        (status = 200, description = "Allowed and currently-selected next play types", body = PlayTypeInfo),
        (status = 409, description = "No game in progress"),
    )
)]
#[get("/nexttype")]
async fn get_next_play_types(appstate: web::Data<AppState>) -> impl Responder {
    println!("Get Next Plays Called");

    lock_game!(appstate, game);

    let next_types = game.allowed_play_types();

    // Convert the OffensiveLineup struct to JSON
    let json_data = serde_json::to_string(&next_types)
        .expect("Error while serializing Alllowed Plays to JSON.");

    // Set the Content-Type header to application/json
    HttpResponse::Ok()
        .content_type("application/json")
        .body(json_data)
}

#[utoipa::path(
    tag = "game",
    request_body(
        content = String,
        content_type = "text/plain",
        description = "PlayType value: Kickoff | Punt | ExtraPoint | FieldGoal | Standard | None"
    ),
    responses(
        (status = 200, description = "Next play type set"),
        (status = 400, description = "Unknown or illegal play type"),
        (status = 409, description = "No game in progress"),
    )
)]
#[post("/nexttype")]
async fn set_next_play_type(appstate: web::Data<AppState>, data: String) -> impl Responder {
    println!("Set Next Play Called");

    println!("Play Type is {}", data);
    let v = PlayType::from_str(&data);
    if v.is_err() {
        return HttpResponse::BadRequest().body("Unknown Type");
    }

    lock_game!(appstate, game);
    let res = game.set_next_play_type(v.unwrap());
    match res {
        Ok(_) => HttpResponse::Ok()
            .content_type("application/json")
            .body("Set"),
        Err(msg) => HttpResponse::BadRequest().body(msg),
    }
}

#[utoipa::path(
    tag = "game",
    request_body(
        content = String,
        content_type = "text/plain",
        description = "Path of the file to save the serialized game to"
    ),
    responses(
        (status = 200, description = "Game saved"),
        (status = 400, description = "Could not save game"),
        (status = 409, description = "No game in progress"),
    )
)]
#[post("/save")]
async fn save_game(appstate: web::Data<AppState>, data: String) -> impl Responder {
    println!("Save Game Called");

    println!("File is {}", data);

    lock_game!(appstate, game);
    let res = game.serialize_struct(data);
    match res {
        Ok(_) => HttpResponse::Ok()
            .content_type("application/json")
            .body("Set"),
        Err(msg) => HttpResponse::BadRequest().body(msg.to_string()),
    }
}

#[utoipa::path(
    tag = "players",
    params(("id" = String, Path, description = "Player ID, e.g. QB-1234")),
    responses(
        (status = 200, description = "Player record; shape varies by position", body = Object),
        (status = 409, description = "No game in progress"),
    )
)]
#[get("/getplayer/{id}")]
async fn get_player(path: web::Path<String>, appstate: web::Data<AppState>) -> impl Responder {
    println!("Get Player Called: {:?}", path);
    let path_param = path.into_inner();
    println!("Get Player Called Inner: {:?}", path_param);

    lock_game!(appstate, game);

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

#[utoipa::path(
    tag = "game",
    params(PlayQueryParams),
    responses(
        (
            status = 200,
            description = "Play history. With `?result=true` each entry contains only \
                           `{result, new_state}`; `?count=N` limits to the last N plays.",
            body = Vec<PlayAndState>
        ),
        (status = 409, description = "No game in progress"),
    )
)]
#[get("/plays")]
async fn get_all_plays(
    appstate: web::Data<AppState>,
    query: web::Query<PlayQueryParams>,
) -> impl Responder {
    lock_game!(appstate, game);
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
    env: GameEnvironment,
    game: Mutex<Option<Game>>,
}

/// Read-only WebSocket endpoint (`GET /game/ws`). On connect the client immediately
/// receives the current game state (as a `GameStarted`-shaped `GameEvent`), then every
/// subsequent `GameEvent` as a JSON text frame. Client commands stay on REST. Returns
/// `409 Conflict` when no game is in progress. See `docs/design/ws-events-architecture.md`.
///
/// Registered via `App::route` (not `#[get]`/utoipa `service`) because a WebSocket upgrade
/// cannot be described by `#[utoipa::path]`, and the utoipa scope's `service` bound requires
/// `OpenApiFactory`. Registering it on the utoipa app *before* the `/game` scope also avoids
/// the scope greedily shadowing the `/game/ws` path.
async fn game_ws(
    req: HttpRequest,
    body: web::Payload,
    appstate: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    // Briefly lock: read the snapshot and mint a receiver, then release the guard before
    // any async WS work (the Mutex guard must not be held across await points).
    let (snapshot, mut rx) = {
        let mut guard = appstate.game.lock().unwrap();
        let game = match guard.as_mut() {
            Some(g) => g,
            None => return Ok(HttpResponse::Conflict().body("No game in progress")),
        };
        (game.state, game.subscribe())
    };

    let (res, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    rt::spawn(async move {
        // Snapshot-then-stream: send the current state first.
        let snapshot_ev = GameEvent::GameStarted { state: snapshot };
        if let Ok(txt) = serde_json::to_string(&snapshot_ev) {
            if session.text(txt).await.is_err() {
                return; // client already gone
            }
        }

        // Multiplex broadcast events and inbound client frames.
        loop {
            tokio::select! {
                event = rx.recv() => match event {
                    Ok(ev) => {
                        if let Ok(txt) = serde_json::to_string(&ev) {
                            if session.text(txt).await.is_err() {
                                break; // client disconnected mid-send
                            }
                        }
                    }
                    Err(RecvError::Lagged(_)) => continue, // skip & resync
                    Err(RecvError::Closed) => break,       // game dropped
                },
                msg = msg_stream.next() => match msg {
                    Some(Ok(Message::Ping(bytes))) => {
                        if session.pong(&bytes).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break, // client closed / stream ended
                    Some(Ok(_)) => {} // read-only: ignore Text/Binary/etc.
                    Some(Err(_)) => break, // protocol error
                },
            }
        }

        let _ = session.close(None).await;
    });

    Ok(res)
}

#[utoipa::path(
    tag = "game",
    request_body = StartGameRequest,
    responses(
        (status = 200, description = "Game started; returns initial game state", body = GameState),
        (status = 404, description = "Unknown team"),
        (status = 409, description = "A game is already in progress"),
    )
)]
#[post("/start")]
async fn start_game(
    appstate: web::Data<AppState>,
    data: web::Json<StartGameRequest>,
) -> impl Responder {
    let req = data.into_inner();
    println!("Start Game: {:?} vs {:?}", req.home, req.away);

    let mut guard = appstate.game.lock().unwrap();
    if guard.is_some() {
        return HttpResponse::Conflict().body("A game is already in progress");
    }

    let game = match Game::create_game(&appstate.env, &req.home, &req.away) {
        Ok(g) => g,
        Err(CreateGameError::UnknownTeam(team)) => {
            return HttpResponse::NotFound().body(format!("Unknown team: {}", team.to_string()))
        }
    };
    let json_data =
        serde_json::to_string(&game.state).expect("Error while serializing State to JSON.");
    *guard = Some(game);

    HttpResponse::Ok()
        .content_type("application/json")
        .body(json_data)
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Statis Pro Football API",
        version = "0.1.0",
        description = "HTTP API for running Statis Pro Football game simulations.\n\n\
                       In addition to the REST endpoints below, the server exposes a \
                       read-only WebSocket at `GET /game/ws` that streams live `GameEvent`s \
                       (see the `GameEvent` schema). utoipa cannot describe WebSocket \
                       upgrades natively, so this endpoint does not appear as a path here; \
                       see the top-level README for a `websocat` usage example."
    ),
    servers(
        (url = "http://127.0.0.1:8080", description = "Local dev server")
    ),
    components(schemas(
        OffenseCall,
        DefenseCall,
        crate::game::standard_play::StandardOffenseCall,
        crate::game::standard_play::StandardDefenseCall,
        crate::game::engine::KickoffOffenseCall,
        crate::game::engine::PuntOffenseCall,
        crate::game::engine::KickoffDefenseCall,
        crate::game::engine::PuntDefenseCall,
        GameEvent,
    ))
)]
struct ApiDoc;

#[actix_web::main]
pub async fn runserver(env: GameEnvironment) -> std::io::Result<()> {
    let app_state = web::Data::new(AppState {
        env,
        game: Mutex::new(None),
    });

    // let game = RefCell::new(game);
    println!("Starting up server....");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:5173") // <- your vue app origin
            .allowed_methods(vec!["GET", "POST"]) // <- allow GET and POST requests
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
            .allowed_header(header::CONTENT_TYPE)
            .max_age(3600)
            .allow_any_origin(); // You can use `allow_origin` to specify a specific origin

        let (app, api) = App::new()
            .into_utoipa_app()
            .openapi(ApiDoc::openapi())
            .app_data(app_state.clone())
            .map(|a| a.wrap(cors))
            .route("/game/ws", web::get().to(game_ws))
            .service(
                scope::scope("/game")
                    .service(start_game)
                    .service(get_game_state)
                    .service(run_play)
                    .service(get_all_plays)
                    .service(save_game)
                    .service(get_next_play_types)
                    .service(set_next_play_type),
            )
            .service(
                scope::scope("/offense")
                    .service(get_offensive_lineup)
                    .service(set_offensive_lineup)
                    .service(set_offense_call),
            )
            .service(
                scope::scope("/defense")
                    .service(get_defensive_lineup)
                    .service(set_defensive_lineup)
                    .service(set_defense_call),
            )
            .service(get_player)
            .service(get_team_players)
            .split_for_parts();

        app.service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", api.clone()))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
