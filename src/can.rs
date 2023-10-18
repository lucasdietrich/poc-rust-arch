use std::sync::Arc;

use crate::utils::Sock;
use serde::Serialize;
use tokio::sync::Mutex;
use std::num::Wrapping;

#[derive(Default, Debug, Serialize, Clone)]
pub struct CanStats {
    pub rx: u32,
    pub tx: u32,
}

pub struct CanConfig {
    pub iface: String,
}

impl Default for CanConfig {
    fn default() -> CanConfig {
        CanConfig {
            iface: "vcan0".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct CanInterface {
    _sock: Sock,
    _n: Wrapping<u8>,
    pub stats: CanStats,
}

#[derive(Debug)]
pub struct CanFrame {
    pub id: u32,
    pub data: [u8; 8],
}

impl CanInterface {
    pub fn new(_config: CanConfig) -> CanInterface {
        CanInterface {
            _sock: Sock::new(),
            _n: Wrapping(0),
            stats: CanStats::default(),
        }
    }

    pub async fn send(&mut self, _frame: CanFrame) {
        self.stats.tx += 1;
        self._n += _frame.id as u8;
    }

    pub async fn recv(&mut self) -> Option<CanFrame> {
        self.stats.rx += 1;
        Some(CanFrame {
            id: 1,
            data: [self._n.0, 0, 0, 0, 0, 0, 0, 0],
        })
    }
}
