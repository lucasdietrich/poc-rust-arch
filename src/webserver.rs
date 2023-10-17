use rocket::serde::{json::Json, Serialize};
use rocket::{log::LogLevel, Build, Config, Rocket, State};

use crate::can::CanStats;
use crate::controller::ControllerStats;
use crate::shared::SharedHandle;

#[derive(Serialize, Default)]
struct Stats {
    pub can: CanStats,
    pub ctrl: ControllerStats,
}

#[get("/stats")]
async fn route_stats(shared: &State<SharedHandle>) -> Json<Stats> {
    // TODO How to retrieve CanInterface stats ?
    let can_stats = shared.can_stats.lock().await;
    let controller_stats = shared.controller_stats.lock().await;
    let stats = Stats {
        can: can_stats.clone(),
        ctrl: controller_stats.clone(),
    };

    Json(stats)
}

#[derive(Serialize, Default)]
struct Response {
    success: bool
}

#[get("/query?<id>")]
async fn route_query(id: u32, shared: &State<SharedHandle>) -> Json<Response> {
    
    // TODO How to query a frame to Controller ?

    let response = Response { success: false };
    Json(response)
}

pub fn webserver(listen: String, port: u16, shared: SharedHandle) -> Rocket<Build> {
    let config = Config {
        workers: 1,
        log_level: LogLevel::Normal,
        port: port,
        address: listen.parse().unwrap(),
        cli_colors: false,
        ..Default::default()
    };

    rocket::custom(config)
        .manage(shared)
        .mount("/", routes![route_stats, route_query])
}
