use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{can::CanStats, controller::ControllerStats};

pub type SharedHandle = Arc<Shared>;

#[derive(Debug)]
pub struct Shared {
    pub can_stats: Arc<Mutex<CanStats>>,
    pub controller_stats: Arc<Mutex<ControllerStats>>,
}

impl Shared {
    pub fn new(can_stats: Arc<Mutex<CanStats>>,
        controller_stats: Arc<Mutex<ControllerStats>>) -> Shared {
        Shared {
            can_stats: can_stats.clone(),
            controller_stats: controller_stats.clone(),
        }
    }
}
