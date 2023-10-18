use serde::Serialize;
use std::{time::Duration};
use tokio::{time::sleep, sync::{mpsc, oneshot}, select, runtime::Runtime};

use crate::can::{CanFrame, CanInterface, CanStats};

#[derive(Debug, Default, Serialize, Clone)]
pub struct ControllerStats {
    pub discovery_count: u32,
}

#[derive(Debug)]
pub struct ControllerState {
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

impl ControllerState {
    pub fn new(iface: CanInterface, config: ControllerConfig) -> ControllerState {
        ControllerState {
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

        let discovery_frame = CanFrame {
            id: 0x123,
            data: [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
        };

        self.iface.send(discovery_frame).await;

        if let Some(frame) = self.iface.recv().await {
            println!("Received frame: {:?}", frame);
        }
    }

    pub async fn query(&mut self, id: u32) -> Option<u32> {
        println!("Querying device: {}", id);

        let query = CanFrame {
            id: id,
            data: [0x00; 8],
        };

        self.iface.recv().await.map(|frame| {
            frame.data[0] as u32
        })
    }
}

pub struct ControllerActor {
    receiver: mpsc::Receiver<ControllerMessage>,
    state: ControllerState,
}

pub enum ControllerMessageType {
    Query(u32),
    GetStats,
}

pub enum ControllerResponse {
    Query(u32),
    GetStats(ControllerStats, CanStats),
}

pub struct ControllerMessage {
    respond_to: oneshot::Sender<ControllerResponse>,
    inner: ControllerMessageType,
}

impl ControllerActor {
    fn new(state: ControllerState, receiver: mpsc::Receiver<ControllerMessage>) -> Self {
        ControllerActor { receiver, state }
    }

    async fn handle_message(&mut self, message: ControllerMessage) {
        match message.inner {
            ControllerMessageType::Query(id) => {
                let id = self.state.query(id).await.unwrap();
                let _ = message.respond_to.send(ControllerResponse::Query(id));
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
    loop {
        select! {
            Some(msg) = actor.receiver.recv() => {
                actor.handle_message(msg).await;
            },
            // receiver can frame
            // handle shutdown
        }

        actor.state.discover().await;
    }
}

#[derive(Clone, Debug)]
pub struct ControllerActorHandler {
    sender: mpsc::Sender<ControllerMessage>,
}

impl ControllerActorHandler {
    pub fn new(rt: &Runtime, state: ControllerState) -> Self {
        let (sender, receiver) = mpsc::channel(8);
        let actor = ControllerActor::new(state, receiver);
        rt.spawn(run_controller_actor(actor));
        Self { sender }
    }

    pub async fn get_unique_id(&self, id: u32) -> u32 {
        let (send, recv) = oneshot::channel();
        let msg = ControllerMessage {
            respond_to: send,
            inner: ControllerMessageType::Query(id),
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