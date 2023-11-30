use std::sync::Arc;

use crate::controller::ControllerActorHandler;

pub type SharedHandle = Arc<Shared>;

#[derive(Debug)]
pub struct Shared {
    pub controller_handler: ControllerActorHandler,
}

impl Shared {
    pub fn new(controller_handler: ControllerActorHandler) -> Shared {
        Shared { controller_handler }
    }
}
