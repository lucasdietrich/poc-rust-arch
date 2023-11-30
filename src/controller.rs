use serde::Serialize;
use std::{time::Duration, num::Wrapping};
use tokio::{time::sleep, sync::{mpsc, oneshot}, select, runtime::Runtime};

use crate::can::{CanFrame, CanInterface, CanStats};

#[derive(Debug, Default, Serialize, Clone)]
pub struct ControllerStats {
    pub discovery_count: u32,
}

#[derive(Debug)]
pub struct Controller {
    pub iface: CanInterface,
    pub stats: ControllerStats,
    pub config: ControllerConfig,
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
    pub fn new(iface: CanInterface, config: ControllerConfig) -> Controller {
        Controller {
            iface,
            stats: ControllerStats::default(),
            config,
        }
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

        if let Some(frame) = self.iface.recv().await {
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

        self.iface.recv().await.map(|frame| {
            frame.data[0] as u32
        })
    }
}

pub struct ControllerActor {
    receiver: mpsc::Receiver<ControllerMessage>,
    state: Controller,
}

#[derive(Debug)]
pub enum ControllerMessageType {
    Query(u32, Option<u32>), // id, timeout_ms
    GetStats,
}

#[derive(Debug)]
pub enum ControllerResponse {
    Query(u32),
    GetStats(ControllerStats, CanStats),
}

#[derive(Debug)]
pub struct ControllerMessage {
    respond_to: oneshot::Sender<ControllerResponse>,
    inner: ControllerMessageType,
}

impl ControllerActor {
    fn new(state: Controller, receiver: mpsc::Receiver<ControllerMessage>) -> Self {
        ControllerActor { receiver, state }
    }

    async fn handle_message(&mut self, message: ControllerMessage) {
        match message.inner {
            ControllerMessageType::Query(id,timeout_ms) => {
                if let Some(id) = self.state.query(id, timeout_ms).await {
                    let _ = message.respond_to.send(ControllerResponse::Query(id));
                } else {
                    let _ = message.respond_to.send(ControllerResponse::Query(0));
                }
            }
            ControllerMessageType::GetStats => {
                let response = ControllerResponse::GetStats(
                    self.state.stats.clone(),
                    self.state.iface.stats.clone(),
                );
                let _ = message.respond_to.send(response);
            }
        }
    }
}

async fn run_controller_actor(mut actor: ControllerActor) {
    let mut counter: Wrapping<u32> = Wrapping(0);
    let mut last_discovery: u32 = 0;
    loop {
        select! {
            Some(msg) = actor.receiver.recv() => {
                println!("Received message: {:?}", msg);
                actor.handle_message(msg).await;
            },
            // Some(msg) = actor.state.iface.recv_frame() => {
            //     println!("Received frame: {:?}", msg);
            // },
            _ = sleep(Duration::from_secs(2)) => {
                println!("Tick");
                counter += 1;
                if counter.0 - last_discovery > actor.state.config.discovery_period {
                    actor.state.discover().await;
                    last_discovery = counter.0;
                }
            },
            // handle shutdown
        }
    }
}

#[derive(Clone, Debug)]
pub struct ControllerActorHandler {
    sender: mpsc::Sender<ControllerMessage>,
}

impl ControllerActorHandler {
    pub fn new(rt: &Runtime, state: Controller) -> Self {
        let (sender, receiver) = mpsc::channel(8);
        let actor = ControllerActor::new(state, receiver);
        rt.spawn(run_controller_actor(actor));
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
        let answer = recv.await.expect("Actor task has been killed");

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
        let answer = recv.await.expect("Actor task has been killed");

        match answer {
            ControllerResponse::GetStats(stats, _) => stats,
            _ => panic!("Unexpected response"),
        }
    }
}