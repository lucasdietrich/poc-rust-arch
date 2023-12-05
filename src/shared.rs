use std::sync::Arc;

use crate::controller::ControllerHandle;

pub type SharedHandle = Arc<Shared>;

#[derive(Debug)]
pub struct Shared {
    pub controller_handle: ControllerHandle,
}

impl Shared {
    pub fn new(controller_handle: ControllerHandle) -> Shared {
        Shared { controller_handle }
    }
}
