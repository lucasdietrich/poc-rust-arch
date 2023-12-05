use rocket::serde::{json::Json, Serialize};
use rocket::{log::LogLevel, Build, Config, Rocket, State};

use crate::alarm::{AlarmAction, AlarmNode};
use crate::can::CanStats;
use crate::controller::{ControllerStats, DeviceNodeAction};
use crate::shared::SharedHandle;

#[derive(Serialize, Default)]
struct Stats {
    pub can: CanStats,
    pub ctrl: ControllerStats,
}

#[get("/dev_action")]
async fn route_dev_action(shared: &State<SharedHandle>) -> Json<Response> {
    let action = AlarmAction::PowerLights(true, true);

    let alarm_handle = shared.controller_handle.query_device(DeviceNodeAction::Alarm(
        AlarmAction::PowerLights(true, true)
    ));

    // let ret = shared.controller_handle.device_handle_action(dev, &action).await;

    Json(Response { id: 0 })
}

#[get("/stats")]
async fn route_stats(shared: &State<SharedHandle>) -> Json<ControllerStats> {
    let stats = shared.controller_handle.get_stats().await;

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
    let id = shared.controller_handle.query(id, timeout).await;
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
        .mount("/", routes![route_stats, route_query, route_dev_action])
}
