use dora_node_api::{
    IntoArrow,
    arrow::{
        array::{AsArray, PrimitiveArray},
        datatypes::Float32Type,
    },
};
use eyre::ContextCompat;

#[derive(Debug, Clone, Default)]
pub struct MotorCommand {
    pub motor_left: f32,
    pub motor_right: f32,
    pub servo: f32,
}

impl IntoArrow for MotorCommand {
    type A = PrimitiveArray<Float32Type>;
    fn into_arrow(self) -> Self::A {
        vec![self.motor_left, self.motor_right, self.servo].into_arrow()
    }
}

impl TryFrom<&dora_node_api::ArrowData> for MotorCommand {
    type Error = eyre::Report;
    fn try_from(value: &dora_node_api::ArrowData) -> Result<Self, Self::Error> {
        let array = value
            .as_primitive_opt::<Float32Type>()
            .context("expected Float32 array")?;
        if array.len() != 3 {
            eyre::bail!("expected 3 elements for MotorCommand");
        }
        Ok(MotorCommand {
            motor_left: array.value(0),
            motor_right: array.value(1),
            servo: array.value(2),
        })
    }
}
