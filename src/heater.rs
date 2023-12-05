use crate::{
    can::CanFrame,
    controller::ControllerAPI,
    device::{DeviceActionTrait, DeviceControllableTrait, DeviceError, DeviceTrait},
};

#[derive(Debug, Default)]
pub struct HeaterNode {
    pub active: bool,
}

#[async_trait]
impl DeviceTrait for HeaterNode {
    async fn handle_frame(&mut self, frame: &CanFrame) -> Result<(), DeviceError> {
        Ok(())
    }
}

pub enum HeaterState {
    Off,
    Comfort,
    Eco,
    AntiFreeze,
}

pub enum HeaterAction {
    SetActive(bool),
    HeaterPower(HeaterState, HeaterState), // left and right heater power
}

impl DeviceActionTrait for HeaterAction {}

#[async_trait]
impl DeviceControllableTrait for HeaterNode {
    type Action = HeaterAction;

    async fn handle_action(
        &mut self,
        api: &dyn ControllerAPI,
        action: &HeaterAction,
    ) -> Result<(), DeviceError> {
        match action {
            HeaterAction::SetActive(active) => {
                self.active = *active;
            }
            HeaterAction::HeaterPower(left, right) => {
                let z = api.query_frame(1, Some(0)).await;
            }
        };

        Ok(())
    }
}
