use std::sync::Arc;

use crate::controller::ControllerHandle;

pub type SharedHandle = Arc<Shared>;

#[derive(Debug)]
pub struct Shared {
    pub controller_handler: ControllerHandle,
}

impl Shared {
    pub fn new(controller_handler: ControllerHandle) -> Shared {
        Shared { controller_handler }
    }
}
