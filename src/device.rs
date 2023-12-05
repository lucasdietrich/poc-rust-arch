use async_trait::async_trait;
use std::{fmt::Debug, time::Instant};
use thiserror::Error;

use crate::{
    can::CanFrame,
    controller::{ControllerAPI, ControllerHandle},
};

#[derive(Error, Debug)]
pub enum DeviceError {
    #[error("Device timed out")]
    Unsupported,
}

#[derive(Debug, Default)]
pub struct Device<D>
where
    D: DeviceTrait,
{
    pub id: u32,
    pub last_seen: Option<Instant>,

    pub specific: D,
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

impl<D> Device<D>
where
    D: DeviceTrait,
{
    fn get_id(&self) -> u32 {
        self.id
    }
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
    D: DeviceTrait + DeviceControllableTrait<Action = A>,
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
pub trait DeviceTrait: Send + Default + Debug {
    // fn get_id(&self) -> u32;
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

pub struct DeviceHandle {
    id: u32,
    ctrl: ControllerHandle,
}

impl DeviceHandle {
    pub fn from<T>(controller_handle: &ControllerHandle, device: &Device<T>) -> DeviceHandle
    where
        T: DeviceTrait,
    {
        DeviceHandle {
            id: device.get_id(),
            ctrl: controller_handle.clone()
        }
    }
}
