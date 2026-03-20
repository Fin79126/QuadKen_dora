mod bno;

use bno::Bno;
use dora_node_api::{
    self, DoraNode, Event, IntoArrow, Parameter, dora_core::config::DataId, init_tracing,
};
use dotenv::dotenv;
use eyre::Context;
use std::collections::BTreeMap;
use std::env;
use tracing::{Level, span};
use tracing::{debug, error, info};
use types::imu::ImuData;

fn main() -> eyre::Result<()> {
    dotenv().ok();
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

fn run(mut node: DoraNode, mut events: dora_node_api::EventStream) -> eyre::Result<()> {
    let debug_mode = env::var("DEBUG")
        .unwrap_or_else(|_| false.to_string())
        .parse::<bool>()
        .unwrap_or(false);

    let mut bno055 = Bno::new(debug_mode).context("Failed to setup BNO055 sensor")?;
    let out_imu = DataId::from("imu_data".to_owned());
    let mut metadata = BTreeMap::new();
    metadata.insert(
        "primitive".to_string(),
        Parameter::String("series".to_string()),
    );

    {
        let span = span!(Level::INFO, "Node Span");
        let _enter = span.enter();
        while let Some(event) = events.recv() {
            match event {
                Event::Input {
                    id,
                    metadata: _,
                    data: _,
                } => match id.as_str() {
                    "tick" => {
                        // ボタン1が押された場合、センサーからデータを取得して表示
                        let euler = bno055.euler_angles().unwrap();
                        let return_data = ImuData {
                            roll: euler.a,
                            pitch: euler.b,
                            yaw: euler.c,
                        };
                        debug!("return_data: {:?}", return_data);
                        node.send_output(
                            out_imu.clone(),
                            metadata.clone(),
                            return_data.into_arrow(),
                        )?;
                    }
                    other => error!("Ignoring unexpected input `{other}`"),
                },
                Event::Stop(_) => {
                    info!("Received stop");
                }
                Event::InputClosed { id } => {
                    info!("Input `{id}` was closed");
                }
                other => error!("Received unexpected input: {other:?}"),
            }
        }
    }

    Ok(())
}
