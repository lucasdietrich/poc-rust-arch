use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{can::CanStats, controller::{ControllerStats, Controller}};

pub type SharedHandle = Arc<Shared>;

#[derive(Debug)]
pub struct Shared {
}

impl Shared {
    pub fn new(
    ) -> Shared {
        Shared {
        }
    }
}
