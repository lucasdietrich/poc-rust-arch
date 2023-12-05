#[macro_use]
extern crate rocket;

mod alarm;
mod can;
mod controller;
mod device;
mod shared;
mod shutdown;
mod utils;
mod webserver;

use std::sync::Arc;

use shutdown::Shutdown;
use tokio::sync::broadcast;

use can::CanConfig;
use controller::{run_controller, ControllerConfig};

fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();

    let (notify_shutdown, _) = broadcast::channel(1);

    let can_config = CanConfig::default();
    let controller_config = ControllerConfig::default();

    let can_iface = can::CanInterface::new(can_config);
    let mut controller = controller::Controller::new(
        &rt,
        can_iface,
        controller_config,
        Shutdown::new(notify_shutdown.subscribe()),
    );
    let controller_handle = controller.get_handle();

    let shared = Arc::new(shared::Shared::new(controller_handle));

    rt.spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");

        info!("CTRL+C received, shutting down...");

        let _ = notify_shutdown.send(());
    });
    
    let h_ctrl = rt.spawn(run_controller(controller));
    let h_web =
        rt.spawn(webserver::webserver("0.0.0.0".to_string(), 8091, shared.clone()).launch());

    // terminate on ctrl-c (handled by rocket)
    let _ = rt.block_on(async move { tokio::join!(h_web, h_ctrl) });
}
