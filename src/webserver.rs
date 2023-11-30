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
async fn route_stats(shared: &State<SharedHandle>) -> Json<ControllerStats> {
    let stats = shared.controller_handler.get_stats().await;

    Json(stats)
}

#[derive(Serialize, Default)]
struct Response {
    id: u32,
}

#[get("/query?<id>&<timeout>")]
async fn route_query(
    id: u32,
    timeout: Option<u32>,
    shared: &State<SharedHandle>,
) -> Json<Response> {
    let id = shared.controller_handler.query(id, timeout).await;
    Json(Response { id })
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
