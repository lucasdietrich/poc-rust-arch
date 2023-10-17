use crate::utils::Sock;
use serde::Serialize;

#[derive(Default, Serialize)]
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

pub struct CanInterface {
    _sock: Sock,
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
            stats: CanStats::default(),
        }
    }

    pub async fn send(&mut self, _frame: CanFrame) {
        self.stats.tx += 1;
    }

    pub async fn recv(&mut self) -> Option<CanFrame> {
        self.stats.rx += 1;
        Some(CanFrame {
            id: 1,
            data: [2, 0, 0, 0, 0, 0, 0, 0],
        })
    }
}
