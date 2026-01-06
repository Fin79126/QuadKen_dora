// #![warn(clippy::single_match)]
use dora_node_api::{self, DoraNode, Event, IntoArrow, dora_core::config::DataId};
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
                "status" => {
                    let status: StatusController = StatusController::try_from(&data)?;
                    debug!("{:?}", status);
                    if status.move_power > 0.5 {
                        trace!("Sending command to start");
                        node.send_output(
                            out_command.clone(),
                            metadata.parameters.clone(),
                            true.into_arrow(),
                        )?;
                    }
                }
                other => error!("Ignoring unexpected input `{other}`"),
            },
            Event::Stop(_) => info!("Received stop"),
            other => error!("Received unexpected input: {other:?}"),
        }
    }

    Ok(())
}
