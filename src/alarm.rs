use crate::{
    can::CanFrame,
    controller::ControllerAPI,
    device::{DeviceActionTrait, DeviceControllableTrait, DeviceError, DeviceTrait},
};

#[derive(Debug, Default)]
pub struct AlarmNode {
    pub active: bool,
    pub triggered_count: u32,
}

#[async_trait]
impl DeviceTrait for AlarmNode {
    async fn handle_frame(&mut self, frame: &CanFrame) -> Result<(), DeviceError> {
        if frame.data[0] & 0x01 != 0 {
            self.triggered_count += 1;
        }

        Ok(())
    }
}

pub enum AlarmAction {
    SetActive(bool),         // set alarm on/off
    PowerLights(bool, bool), // set front and rear lights on/off
}

impl DeviceActionTrait for AlarmAction {}

#[async_trait]
impl DeviceControllableTrait for AlarmNode {
    type Action = AlarmAction;

    async fn handle_action(
        &mut self,
        api: &dyn ControllerAPI,
        action: &AlarmAction,
    ) -> Result<(), DeviceError> {
        match action {
            AlarmAction::SetActive(active) => {
                self.active = *active;
            }
            AlarmAction::PowerLights(front, rear) => {
                let z = api.query_frame(1, Some(0)).await;
            }
        };

        Ok(())
    }
}
