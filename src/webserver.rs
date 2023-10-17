use rocket::serde::{json::Json, Serialize};
use rocket::{log::LogLevel, Build, Config, Rocket, State};

use crate::can::CanStats;
use crate::shared::SharedHandle;

#[derive(Serialize, Default)]
struct Stats {
    pub can: CanStats,
}

#[get("/stats")]
fn route_stats(_shared: &State<SharedHandle>) -> Json<Stats> {
    // How to retrieve CanInterface stats ?
    let stats = Stats::default();
    Json(stats)
}

pub fn webserver(listen: String, port: u16, shared: SharedHandle) -> Rocket<Build> {
    let config = Config {
        workers: 1,
        log_level: LogLevel::Critical,
        port: port,
        address: listen.parse().unwrap(),
        cli_colors: false,
        ..Default::default()
    };

    rocket::custom(config)
        .manage(shared)
        .mount("/", routes![route_stats])
}
