use dora_node_api::{
    self, DoraNode, Event, EventStream, IntoArrow, dora_core::config::DataId, init_tracing,
};
use eyre::Context;
use std::io::Read;
use std::net::{TcpListener, UdpSocket};
use tracing::{Level, error, info, span};
use types::controller::StatusController;
pub fn bits_to_status(bits: u16) -> StatusController {
    // ビットチェック用クロージャ
    let is_on = |b: u16| -> bool { bits & (1 << b) != 0 };

    // ----------------------------
    // WASD → 角度
    // ----------------------------
    let mut angle_horizontal: f32 = 0.0;
    let mut angle_vertical: f32 = 0.0;

    if is_on(1) {
        // A
        angle_horizontal -= 1.0;
    }
    if is_on(3) {
        // D
        angle_horizontal += 1.0;
    }
    if is_on(0) {
        // W
        angle_vertical += 1.0;
    }
    if is_on(2) {
        // S
        angle_vertical -= 1.0;
    }

    // 正規化（斜め入力で1を超えないように）
    let len = (angle_horizontal.powi(2) + angle_vertical.powi(2)).sqrt();
    if len > 1.0 {
        angle_horizontal /= len;
        angle_vertical /= len;
    }

    // ----------------------------
    // move_power（前進系）
    // ----------------------------
    let mut move_power = 0.0;
    if is_on(4) {
        // Q
        move_power += 1.0;
    }
    if is_on(5) {
        // E
        move_power -= 1.0;
    }

    // ----------------------------
    // roll（回転）
    // ----------------------------
    let mut roll_power = 0.0;
    if is_on(6) {
        // R
        roll_power += 1.0;
    }
    if is_on(7) {
        // F
        roll_power -= 1.0;
    }

    // ----------------------------
    // そのままボタンも保持
    // ----------------------------
    StatusController {
        move_power,
        angle_horizontal,
        angle_vertical,
        roll_power,
        buttons: bits,
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
    let out_status = DataId::from("status".to_owned());
    let udp = UdpSocket::bind("0.0.0.0:5000")?;
    udp.set_nonblocking(true)?;
    // TCP
    let listener = TcpListener::bind("0.0.0.0:6000")?;
    let (mut tcp, _) = listener.accept()?;
    tcp.set_nonblocking(true)?;

    let mut buf = [0u8; 2];
    let mut tcp_buf = [0u8; 1];
    let mut latest_keys: Option<u16> = None;
    {
        let span = span!(Level::INFO, "Input TEST");
        let _enter = span.enter();
        // use a fixed seed for reproducibility (we use this node's output in integration tests)
        while let Some(event) = events.recv() {
            match event {
                Event::Input {
                    id,
                    metadata,
                    data: _,
                } => match id.as_str() {
                    "tick" => {
                        while let Ok(n) = tcp.read(&mut tcp_buf)
                            && n > 0
                        {
                            match tcp_buf[0] {
                                1 => {
                                    info!("start received");
                                }
                                2 => {
                                    info!("shutdown received, exiting");
                                    // node.send_output(
                                    //     out_shutdown.clone(),
                                    //     metadata.parameters.clone(),
                                    //     true.into_arrow(),
                                    // )?;
                                    return Ok(());
                                }
                                _ => {}
                            }
                        }

                        // UDP drain
                        while let Ok((n, _)) = udp.recv_from(&mut buf) {
                            if n == 2 {
                                latest_keys = Some(u16::from_le_bytes(buf));
                            }
                        }
                        if let Some(keys) = latest_keys.take() {
                            let status = bits_to_status(keys);
                            node.send_output(
                                out_status.clone(),
                                metadata.parameters.clone(),
                                status.into_arrow(),
                            )?;
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
