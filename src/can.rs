use crate::utils::Sock;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::num::Wrapping;
use tokio::time::Duration;

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
    buf: Vec<CanFrameLoopback>,
    pub stats: CanStats,
}

#[derive(Debug)]
pub struct CanFrame {
    pub id: u32,
    pub data: [u8; 8],
}

#[derive(Debug)]
struct CanFrameLoopback {
    frame: CanFrame,
    push_timestamp: DateTime<Utc>,
}

const DELAY: u64 = 750;

impl CanInterface {
    pub fn new(_config: CanConfig) -> CanInterface {
        CanInterface {
            _sock: Sock::new(),
            _n: Wrapping(0),
            buf: Vec::new(),
            stats: CanStats::default(),
        }
    }

    pub async fn send(&mut self, frame: CanFrame) {
        self.stats.tx += 1;
        self._n += frame.data[0] as u8;
        self.buf.push(CanFrameLoopback {
            frame,
            push_timestamp: Utc::now(),
        });
    }

    pub async fn recv(&mut self, loopback: bool) -> Option<CanFrame> {
        let now = Utc::now();

        if loopback {
            if let Some(lp_frame) = self.buf.get(0) {
                if lp_frame.push_timestamp + Duration::from_millis(DELAY) < now {
                    let mut frame = self.buf.pop().unwrap().frame;
                    frame.data[0] = frame.data[0].wrapping_add(self._n.0);
                    self.stats.rx += 1;
                    return Some(frame);
                }
            }
        }

        None
    }
}
