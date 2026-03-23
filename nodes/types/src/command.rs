use std::vec;

use dora_node_api::{
    IntoArrow,
    arrow::{
        array::{AsArray, PrimitiveArray},
        datatypes::{Float32Type, UInt8Type, UInt16Type},
    },
};
use eyre::ContextCompat;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BattCommand {
    pub servo: [u16; 4], // 0-180など
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MainCommand {
    Setup,
    GetStatus,
    AttachServo,
    DetachServo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Status {
    pub heartbeat: u32,
}

impl IntoArrow for Status {
    type A = PrimitiveArray<Float32Type>;
    fn into_arrow(self) -> Self::A {
        vec![self.heartbeat as f32].into_arrow()
    }
}

impl TryFrom<&dora_node_api::ArrowData> for Status {
    type Error = eyre::Report;
    fn try_from(value: &dora_node_api::ArrowData) -> Result<Self, Self::Error> {
        let array = value
            .as_primitive_opt::<Float32Type>()
            .context("expected Float32 array")?;
        if array.len() != 1 {
            eyre::bail!("expected 1 element for Status");
        }
        Ok(Status {
            heartbeat: array.value(0) as u32,
        })
    }
}

impl IntoArrow for MainCommand {
    type A = PrimitiveArray<UInt8Type>;
    fn into_arrow(self) -> Self::A {
        let code: u8 = match self {
            MainCommand::Setup => 0,
            MainCommand::GetStatus => 1,
            MainCommand::AttachServo => 2,
            MainCommand::DetachServo => 3,
        };
        code.into_arrow()
    }
}

impl TryFrom<&dora_node_api::ArrowData> for MainCommand {
    type Error = eyre::Report;
    fn try_from(value: &dora_node_api::ArrowData) -> Result<Self, Self::Error> {
        let array = value
            .as_primitive_opt::<UInt8Type>()
            .context("expected UInt8 array")?;
        if array.len() != 1 {
            eyre::bail!("expected 1 element for MainCommand");
        }
        let code = array.value(0);
        let command = match code {
            0 => MainCommand::Setup,
            1 => MainCommand::GetStatus,
            2 => MainCommand::AttachServo,
            3 => MainCommand::DetachServo,
            _ => eyre::bail!("invalid command code: {code}"),
        };
        Ok(command)
    }
}

impl IntoArrow for BattCommand {
    type A = PrimitiveArray<UInt16Type>;
    fn into_arrow(self) -> Self::A {
        vec![self.servo[0], self.servo[1], self.servo[2], self.servo[3]].into_arrow()
    }
}

impl TryFrom<&dora_node_api::ArrowData> for BattCommand {
    type Error = eyre::Report;
    fn try_from(value: &dora_node_api::ArrowData) -> Result<Self, Self::Error> {
        let array = value
            .as_primitive_opt::<UInt16Type>()
            .context("expected UInt16 array")?;
        if array.len() != 4 {
            eyre::bail!("expected 4 elements for BattCommand");
        }
        Ok(BattCommand {
            servo: [
                array.value(0),
                array.value(1),
                array.value(2),
                array.value(3),
            ],
        })
    }
}
