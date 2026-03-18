use dora_node_api::{self, dora_core::config::DataId, DoraNode, Event, IntoArrow};
use tracing::{debug, error, info, trace};
use types::{controller::StatusController, imu::ImuData};

fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt()
        // .with_ansi(false)
        .with_env_filter("debug")
        .try_init()
        .ok(); // すでに初期化されている場合は無視

    let out_command = DataId::from("command".to_owned());
    let (mut node, mut events) = DoraNode::init_from_env()?;

    while let Some(event) = events.recv() {
        match event {
            Event::Input { id, metadata, data } => match id.as_str() {
                "imu_data" => {
                    let imu: ImuData = ImuData::try_from(&data)?;
                    debug!("{:?}", imu);
                }
                "shutdown" => {
                    info!("shutdown received, exiting");
                    return Ok(());
                }
                other => error!("Ignoring unexpected input `{other}`"),
            },
            Event::Stop(_) => info!("Received stop"),
            other => error!("Received unexpected input: {other:?}"),
        }
    }

    Ok(())
}
