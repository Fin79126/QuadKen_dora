mod bno;

use bno::Bno;
use dora_node_api::{self, DoraNode, Event, IntoArrow, dora_core::config::DataId};
use dotenv::dotenv;
use eyre::Context;
use std::env;
use tracing::{debug, error, info};
use types::ImuData;

fn main() -> eyre::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt()
        // .with_ansi(false)
        .with_env_filter("debug")
        .try_init()
        .ok(); // すでに初期化されている場合は無視

    let debug_mode = env::var("DEBUG")
        .unwrap_or_else(|_| false.to_string())
        .parse::<bool>()
        .unwrap_or(false);
    info!("DEBUG mode: {}", debug_mode);
    let mut bno055 = Bno::new(debug_mode).context("Failed to setup BNO055 sensor")?;
    info!("BNO055 sensor initialized");

    let (mut node, mut events) = DoraNode::init_from_env()?;
    let out_imu = DataId::from("imu_data".to_owned());
    info!("Node initialized");

    while let Some(event) = events.recv() {
        // info!("Event received: {:?}", event);
        match event {
            Event::Input { id, metadata, data } => match id.as_str() {
                "keys" => {
                    let received_keys: u16 =
                        TryFrom::try_from(&data).context("expected u16 message")?;
                    debug!("get keys : {received_keys:016b}");
                    if received_keys & 0x0001 != 0 {
                        // ボタン1が押された場合、センサーからデータを取得して表示
                        let euler = bno055.euler_angles().unwrap();
                        debug!(
                            "Euler Angles - X: {:.2}, Y: {:.2}, Z: {:.2}",
                            euler.a, euler.b, euler.c
                        );
                        let return_data = ImuData {
                            roll: euler.a,
                            pitch: euler.b,
                            yaw: euler.c,
                        };
                        node.send_output(
                            out_imu.clone(),
                            metadata.parameters.clone(),
                            return_data.into_arrow(),
                        )?;
                    }
                }
                "shutdown" => {
                    let shutdown: bool =
                        TryFrom::try_from(&data).context("expected bool message")?;
                    if shutdown {
                        info!("shutdown received, exiting");
                        return Ok(());
                    }
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

    Ok(())
}
