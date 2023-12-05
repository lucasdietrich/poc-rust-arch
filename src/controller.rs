use serde::Serialize;
use std::{num::Wrapping, time::Duration};
use tokio::{
    runtime::Runtime,
    select,
    sync::{mpsc, oneshot},
    time::sleep,
};

use crate::{
    alarm::AlarmNode,
    can::{CanFrame, CanInterface, CanStats},
    device::{
        Device, DeviceAction, DeviceActionTrait, DeviceControllableTrait, DeviceError, DeviceTrait,
    },
    shutdown::Shutdown,
};

#[derive(Debug, Default, Serialize, Clone)]
pub struct ControllerStats {
    pub discovery_count: u32,
}

#[derive(Debug)]
pub(crate) struct Controller {
    iface: CanInterface,
    stats: ControllerStats,
    config: ControllerConfig,

    shutdown: Shutdown,
    receiver: mpsc::Receiver<ControllerMessage>,
    handle: ControllerHandle,

    dev_alarm: Device<AlarmNode>,
}

#[derive(Debug)]
pub struct ControllerConfig {
    pub discovery_period: u32, // in seconds
}

impl Default for ControllerConfig {
    fn default() -> ControllerConfig {
        ControllerConfig {
            discovery_period: 5,
        }
    }
}

impl Controller {
    pub fn new(
        rt: &Runtime,
        iface: CanInterface,
        config: ControllerConfig,
        shutdown: Shutdown,
    ) -> Controller {
        let (sender, receiver) = mpsc::channel(8);

        Controller {
            iface,
            stats: ControllerStats::default(),
            config,
            shutdown,
            receiver,
            handle: ControllerHandle::new(rt, sender),
            dev_alarm: Device::<AlarmNode> {
                id: 0x123,
                ..Default::default()
            },
        }
    }

    pub fn get_handle(&mut self) -> ControllerHandle {
        self.handle.clone()
    }

    async fn handle_message(&mut self, message: ControllerMessage) {
        match message.inner {
            ControllerMessageType::Query(id, timeout_ms) => {
                if let Some(id) = self.query(id, timeout_ms).await {
                    let _ = message.respond_to.send(ControllerResponse::Query(id));
                } else {
                    let _ = message.respond_to.send(ControllerResponse::Query(0));
                }
            }
            ControllerMessageType::GetStats => {
                let response =
                    ControllerResponse::GetStats(self.stats.clone(), self.iface.stats.clone());
                let _ = message.respond_to.send(response);
            }
            ControllerMessageType::QueryDevice(action) => {}
        }
    }

    async fn handle_frame(&mut self, frame: CanFrame) {
        // if frame.id == self.dev_alarm.id {
        //     self.dev_alarm.handle_frame(&frame).await;
        // }
    }

    async fn discover(&mut self) {
        println!(
            "Discovering devices... (count: {})",
            self.stats.discovery_count
        );
        self.stats.discovery_count += 1;

        // let discovery_frame = CanFrame {
        //     id: 0x123,
        //     data: [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
        // };

        // self.iface.send(discovery_frame).await;

        if let Some(frame) = self.iface.recv(true).await {
            println!("Received frame: {:?}", frame);
        }
    }

    pub async fn query(&mut self, id: u32, timeout_ms: Option<u32>) -> Option<u32> {
        println!("Querying device: {} timeout {:?}", id, timeout_ms);

        let query = CanFrame {
            id: id,
            data: [0xFF; 8],
        };
        self.iface.send(query).await;

        self.iface
            .recv(true)
            .await
            .map(|frame| frame.data[0] as u32)
    }
}

pub enum ControllerMessageType {
    Query(u32, Option<u32>), // id, timeout_ms
    GetStats,
    QueryDevice(Box<dyn DeviceActionTrait>),
}

#[derive(Debug)]
pub enum ControllerResponse {
    Query(u32),
    GetStats(ControllerStats, CanStats),
}

pub struct ControllerMessage {
    respond_to: oneshot::Sender<ControllerResponse>,
    inner: ControllerMessageType,
}

pub(crate) async fn run_controller(mut ctrl: Controller) {
    let mut counter: Wrapping<u32> = Wrapping(0);
    let mut last_discovery: u32 = 0;
    loop {
        select! {
            Some(msg) = ctrl.receiver.recv() => {
                // println!("Received message: {:?}", msg);
                ctrl.handle_message(msg).await;
            },
            Some(msg) = ctrl.iface.recv(false) => {
                println!("Received frame: {:?}", msg);
                ctrl.handle_frame(msg).await;
            },
            _ = sleep(Duration::from_secs(2)) => {
                println!("Tick");
                counter += 1;
                if counter.0 - last_discovery > ctrl.config.discovery_period {
                    ctrl.discover().await;
                    last_discovery = counter.0;
                }
            },
            _ = ctrl.shutdown.recv() => {
                println!("Shutting down controller");
                break;
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct ControllerHandle {
    sender: mpsc::Sender<ControllerMessage>,
}

impl ControllerHandle {
    pub fn new(rt: &Runtime, sender: mpsc::Sender<ControllerMessage>) -> Self {
        Self { sender }
    }

    pub async fn query(&self, id: u32, timeout_ms: Option<u32>) -> u32 {
        let (send, recv) = oneshot::channel();
        let msg = ControllerMessage {
            respond_to: send,
            inner: ControllerMessageType::Query(id, timeout_ms),
        };

        // Ignore send errors. If this send fails, so does the
        // recv.await below. There's no reason to check the
        // failure twice.
        let _ = self.sender.send(msg).await;
        let answer = recv.await.expect("Controller task has been killed");

        match answer {
            ControllerResponse::Query(answer) => answer,
            _ => panic!("Unexpected response"),
        }
    }

    pub async fn get_stats(&self) -> ControllerStats {
        let (send, recv) = oneshot::channel();
        let msg = ControllerMessage {
            respond_to: send,
            inner: ControllerMessageType::GetStats,
        };

        // Ignore send errors. If this send fails, so does the
        // recv.await below. There's no reason to check the
        // failure twice.
        let _ = self.sender.send(msg).await;
        let answer = recv.await.expect("Controller task has been killed");

        match answer {
            ControllerResponse::GetStats(stats, _) => stats,
            _ => panic!("Unexpected response"),
        }
    }

    pub async fn device_handle_action<A: DeviceActionTrait>(
        &mut self,
        dev: &mut dyn DeviceControllableTrait<Action = A>,
        action: &A,
    ) -> Result<(), DeviceError> {
        dev.handle_action(self, action).await
    }
}

#[async_trait]
pub trait ControllerAPI: Send + Sync {
    async fn query(&self, id: u32, timeout_ms: Option<u32>) -> u32;
}

#[async_trait]
impl ControllerAPI for ControllerHandle {
    async fn query(&self, id: u32, timeout_ms: Option<u32>) -> u32 {
        ControllerHandle::query(self, id, timeout_ms).await
    }
}

// #[derive(Clone, Debug)]
// pub struct DeviceHandle<D: DeviceControllableTrait> {
//     dev: &mut D,
// }
