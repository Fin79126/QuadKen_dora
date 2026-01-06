use dora_node_api::{
    IntoArrow,
    arrow::{
        array::{AsArray, PrimitiveArray},
        datatypes::Float32Type,
    },
};
use eyre::ContextCompat;

#[derive(Debug, Clone)]
pub struct ImuData {
    pub roll: f32,
    pub pitch: f32,
    pub yaw: f32,
}

impl IntoArrow for ImuData {
    type A = PrimitiveArray<Float32Type>;
    fn into_arrow(self) -> Self::A {
        vec![self.roll, self.pitch, self.yaw].into_arrow()
    }
}

impl TryFrom<&dora_node_api::ArrowData> for ImuData {
    type Error = eyre::Report;
    fn try_from(value: &dora_node_api::ArrowData) -> Result<Self, Self::Error> {
        let array = value
            .as_primitive_opt::<Float32Type>()
            .context("expected Float32 array")?;
        if array.len() != 3 {
            eyre::bail!("expected 3 elements for ImuData");
        }
        Ok(ImuData {
            roll: array.value(0),
            pitch: array.value(1),
            yaw: array.value(2),
        })
    }
}

// めんどいからやめた。後で必要になったら自作struct作って孤児ルール解除
// impl IntoArrow for vec::Vec<ImuData> {
//     type A = PrimitiveArray<Float32Type>;
//     fn into_arrow(self) -> Self::A {
//         let mut rolls = Vec::with_capacity(self.len());
//         let mut pitchs = Vec::with_capacity(self.len());
//         let mut yaws = Vec::with_capacity(self.len());
//         for imu in self {
//             rolls.push(imu.roll);
//             pitchs.push(imu.pitch);
//             yaws.push(imu.yaw);
//         }
//         let mut array_builder = PrimitiveArray::<Float32Type>::builder(rolls.len() * 3);
//         for i in 0..rolls.len() {
//             array_builder.append_value(rolls[i]);
//             array_builder.append_value(pitchs[i]);
//             array_builder.append_value(yaws[i]);
//         }
//         array_builder.finish()
//     }
// }
