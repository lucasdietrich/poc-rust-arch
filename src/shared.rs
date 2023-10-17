use std::sync::Arc;

pub type SharedHandle = Arc<Shared>;

#[derive(Debug)]
pub struct Shared {}

impl Shared {
    pub fn new() -> Shared {
        Shared {}
    }
}
