use async_trait::async_trait;
use std::{time::Instant, fmt::Debug};
use thiserror::Error;

use crate::{can::CanFrame, controller::ControllerAPI};

#[derive(Error, Debug)]
pub enum DeviceError {
    #[error("Device timed out")]
    Unsupported,
}

#[derive(Debug, Default)]
pub struct Device<D>
where
    D: DeviceTrait + Debug,
{   
    pub id: u32,
    pub last_seen: Option<Instant>,

    pub specific: D,
}

#[async_trait]
impl<D> DeviceTrait for Device<D>
where
    D: DeviceTrait + Debug,
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

// #[async_trait]
// impl<D> DeviceControllableTrait for Device<D>
// where
//     D: DeviceTrait + DeviceControllableTrait<Action = DeviceAction>,
// {
//     type Action = DeviceAction;

//     async fn handle_action(
//         &mut self,
//         api: &dyn ControllerAPI,
//         action: &DeviceAction,
//     ) -> Result<(), DeviceError> {
//         match action {
//             DeviceAction::Reset => {
//                 let _ = api.query(1, Some(0)).await;
//             }
//         };

//         Ok(())
//     }
// }

#[async_trait]
impl<D, A> DeviceControllableTrait for Device<D>
where
    D: DeviceTrait + DeviceControllableTrait<Action = A> + Debug,
    A: DeviceActionTrait,
{
    type Action = A;

    async fn handle_action(
        &mut self,
        api: &dyn ControllerAPI,
        action: &Self::Action,
    ) -> Result<(), DeviceError> {
        self.specific.handle_action(api, action).await
    }
}

#[async_trait]
pub trait DeviceTrait: Send + Default {
    async fn handle_frame(&mut self, frame: &CanFrame) -> Result<(), DeviceError>;
}

pub trait DeviceActionTrait: Sync + Send {
    // fn get_action(&self) -> Self {
    //     self
    // }
}

#[async_trait]
pub trait DeviceControllableTrait: Send {
    type Action: DeviceActionTrait;

    async fn handle_action(
        &mut self,
        api: &dyn ControllerAPI,
        action: &Self::Action,
    ) -> Result<(), DeviceError>;
}
