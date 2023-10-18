use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{can::CanStats, controller::{ControllerStats, ControllerState, ControllerActorHandler}};

pub type SharedHandle = Arc<Shared>;

#[derive(Debug)]
pub struct Shared {
    pub controller_handler: ControllerActorHandler,
}

impl Shared {
    pub fn new(
        controller_handler: ControllerActorHandler,
    ) -> Shared {
        Shared {
            controller_handler
        }
    }
}
