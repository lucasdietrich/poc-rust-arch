#[macro_use]
extern crate rocket;

mod can;
mod controller;
mod shared;
mod utils;
mod webserver;

use std::sync::Arc;

use can::CanConfig;
use controller::ControllerConfig;

fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();

    let can_config = CanConfig::default();
    let can_iface = can::CanInterface::new(can_config);
    let can_stats = can_iface.stats.clone();

    let controller_config = ControllerConfig::default();
    let mut controller = controller::Controller::new(can_iface, controller_config);
    let ctrl_stats = controller.stats.clone();

    let shared = Arc::new(shared::Shared::new(can_stats, ctrl_stats));

    // My controller has a main task which runs forever (periodic polling and stuff)
    rt.spawn(async move {
        controller.run().await;
    });

    let h_web =
        rt.spawn(webserver::webserver("0.0.0.0".to_string(), 8091, shared.clone()).launch());

    // terminate on ctrl-c (handled by rocket)
    let _ = rt.block_on(async move { tokio::join!(h_web) });
}
