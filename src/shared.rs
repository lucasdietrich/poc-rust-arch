use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{can::CanStats, controller::{ControllerStats, Controller}};

pub type SharedHandle = Arc<Shared>;

#[derive(Debug)]
pub struct Shared {
    // stats
    pub can_stats: Arc<Mutex<CanStats>>,
    pub controller_stats: Arc<Mutex<ControllerStats>>,

    // controller structure
    // pub controller: Arc<Mutex<Controller>>,
}

impl Shared {
    pub fn new(
        can_stats: Arc<Mutex<CanStats>>,
        controller_stats: Arc<Mutex<ControllerStats>>,
    ) -> Shared {
        Shared {
            can_stats: can_stats.clone(),
            controller_stats: controller_stats.clone(),
        }
    }
}
