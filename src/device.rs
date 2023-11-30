use async_trait::async_trait;
use std::time::Instant;
use thiserror::Error;

use crate::{can::CanFrame, controller::ControllerAPI};

#[derive(Error, Debug)]
pub enum DeviceError {
    #[error("Device timed out")]
    Unsupported,
}

struct Device<D>
where
    D: DeviceTrait,
{
    last_seen: Option<Instant>,
    specific: D,
}

#[async_trait]
impl<D> DeviceTrait for Device<D>
where
    D: DeviceTrait,
{
    async fn handle_frame(&mut self, frame: &CanFrame) -> Result<(), DeviceError> {
        self.last_seen = Some(Instant::now());

        self.specific.handle_frame(frame).await
    }

    // async fn handle_action(&mut self, action: &dyn DeviceAction<Self>) -> Result<(), DeviceError> {
    //     self.specific.handle_action(action).await
    // }
}

pub enum DeviceAction {
    Reset,
}

impl DeviceActionTrait for DeviceAction {}

#[async_trait]
impl<D> DeviceControllableTrait<DeviceAction> for Device<D>
where
    D: DeviceTrait,
{
    async fn handle_action(
        &mut self,
        api: &dyn ControllerAPI,
        action: &DeviceAction,
    ) -> Result<(), DeviceError> {
        match action {
            DeviceAction::Reset => {
                let _ = api.query(1, Some(0)).await;
            }
        };

        Ok(())
    }
}

// #[async_trait]
// impl<D, A> DeviceControllableTrait<A> for Device<D>
// where
//     D: DeviceTrait + DeviceControllableTrait<A>,
//     A: DeviceActionTrait,
// {
//     async fn handle_action(
//         &mut self,
//         api: &dyn ControllerAPI,
//         action: &A,
//     ) -> Result<(), DeviceError> {
//         self.specific.handle_action(api, action).await
//     }
// }


#[async_trait]
pub trait DeviceTrait: Send {
    async fn handle_frame(&mut self, frame: &CanFrame) -> Result<(), DeviceError>;
}

pub trait DeviceActionTrait: Sync
{
    fn get_action(&self) -> &Self {
        self
    }
}

#[async_trait]
pub trait DeviceControllableTrait<A>: Send
    where A: DeviceActionTrait
{
    async fn handle_action(
        &mut self,
        api: &dyn ControllerAPI,
        action: &A,
    ) -> Result<(), DeviceError>;
}
