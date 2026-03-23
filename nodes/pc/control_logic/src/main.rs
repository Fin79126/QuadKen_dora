use dora_node_api::{
    self, DoraNode, Event, EventStream, IntoArrow, Parameter, dora_core::config::DataId,
    init_tracing,
};
use eyre::Context;
use std::collections::BTreeMap;
use tracing::{Level, debug, error, info, span};
use types::{BattCommand, ImuData, StatusController};

fn compute_batt_command(controller: &StatusController, imu: &ImuData) -> BattCommand {
    // ここに制御ロジックを実装する
    // これはあくまでダミーの例です
    BattCommand {
        servo: [
            (controller.angle_horizontal * 1000.0) as u16,
            (controller.angle_vertical * 1000.0) as u16,
            (imu.roll * 1000.0) as u16,
            (imu.pitch * 1000.0) as u16,
        ],
    }
}

fn main() -> eyre::Result<()> {
    let (node, events) = DoraNode::init_from_env()?;
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()
        .context("failed to build tokio runtime")?;
    let rt_guard = rt.enter();
    let tracing_guard =
        init_tracing(&node.id().clone(), node.dataflow_id()).context("failed to init tracing")?;
    std::thread::sleep(std::time::Duration::from_millis(100));

    let result = run(node, events);

    drop(tracing_guard);
    drop(rt_guard);
    result
}

fn run(mut node: DoraNode, mut events: EventStream) -> eyre::Result<()> {
    let out_command = DataId::from("command".to_owned());
    let mut metadata = BTreeMap::new();
    metadata.insert(
        "primitive".to_string(),
        Parameter::String("series".to_string()),
    );
    let mut latest_imu: Option<ImuData> = None;
    let mut latest_controller: Option<StatusController> = None;

    {
        let span = span!(Level::INFO, "Input TEST");
        let _enter = span.enter();
        // use a fixed seed for reproducibility (we use this node's output in integration tests)
        while let Some(event) = events.recv() {
            match event {
                Event::Input {
                    id,
                    metadata: _,
                    data,
                } => match id.as_str() {
                    "imu_data" => {
                        latest_imu = Some(ImuData::try_from(&data)?);
                        debug!("{:?}", latest_imu);
                    }
                    "status" => {
                        latest_controller = Some(StatusController::try_from(&data)?);
                        debug!("{:?}", latest_controller);
                    }
                    "tick" => {
                        // ここで制御ロジックを実行
                        if let (Some(imu), Some(controller)) = (&latest_imu, &latest_controller) {
                            // 制御ロジックの実装例（ダミー）
                            let command = compute_batt_command(controller, imu);
                            debug!("Computed command: {:?}", command);
                            node.send_output(
                                out_command.clone(),
                                metadata.clone(),
                                command.into_arrow(),
                            )?;
                        } else {
                            debug!("Waiting for both imu and controller data...");
                        }
                    }

                    other => error!("Ignoring unexpected input `{other}`"),
                },
                Event::Stop(_) => info!("Received stop"),
                other => error!("Received unexpected input: {other:?}"),
            }
        }
    }
    Ok(())
}
