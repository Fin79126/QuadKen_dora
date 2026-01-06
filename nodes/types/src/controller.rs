use dora_node_api::{
    IntoArrow,
    arrow::{
        array::{AsArray, PrimitiveArray},
        datatypes::Float32Type,
    },
};
use eyre::ContextCompat;

#[derive(Debug, Clone, Default)]
pub struct StatusController {
    // pub is_active: bool,
    // pub error_code: u32,
    pub move_power: f32,
    pub angle_horizontal: f32,
    pub angle_vertical: f32,
    pub roll_power: f32,
}

impl IntoArrow for StatusController {
    type A = PrimitiveArray<Float32Type>;
    fn into_arrow(self) -> Self::A {
        vec![
            self.move_power,
            self.angle_horizontal,
            self.angle_vertical,
            self.roll_power,
        ]
        .into_arrow()
    }
}

impl TryFrom<&dora_node_api::ArrowData> for StatusController {
    type Error = eyre::Report;
    fn try_from(value: &dora_node_api::ArrowData) -> Result<Self, Self::Error> {
        let array = value
            .as_primitive_opt::<Float32Type>()
            .context("expected Float32 array")?;
        if array.len() != 4 {
            eyre::bail!("expected 4 elements for StatusController");
        }
        Ok(StatusController {
            move_power: array.value(0),
            angle_horizontal: array.value(1),
            angle_vertical: array.value(2),
            roll_power: array.value(3),
        })
    }
}
