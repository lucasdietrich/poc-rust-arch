use serde::Serialize;
use std::{time::Duration};
use tokio::{time::sleep};

use crate::can::{CanFrame, CanInterface};

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

    pub async fn run(&mut self) {
        loop {
            self.discover().await;
            sleep(Duration::from_secs(self.config.discovery_period as u64)).await;
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

    pub async fn query(&mut self, id: u32) -> Option<CanFrame> {
        println!("Querying device: {}", id);

        let query = CanFrame {
            id: id,
            data: [0x00; 8],
        };

        self.iface.send(query).await;
        let response = self.iface.recv().await;
        
        response
    }
}
