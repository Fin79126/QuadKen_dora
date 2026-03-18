// #![warn(clippy::single_match)]
use dora_node_api::{self, DoraNode, Event, IntoArrow, dora_core::config::DataId};
use std::io::Read;
use std::net::{TcpListener, UdpSocket};
use tracing::{debug, error, info, trace};
use types::controller::StatusController;

fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt()
        // .with_ansi(false)
        .with_env_filter("debug")
        .try_init()
        .ok(); // すでに初期化されている場合は無視

    let out_shutdown = DataId::from("shutdown".to_owned());
    let out_status = DataId::from("status".to_owned());
    let (mut node, mut events) = DoraNode::init_from_env()?;
    debug!("{}", node.dataflow_id());
    info!("Node initialized");
    let udp = UdpSocket::bind("0.0.0.0:5000")?;
    udp.set_nonblocking(true)?;
    info!("UDP socket bound");
    // TCP
    let listener = TcpListener::bind("0.0.0.0:6000")?;
    let (mut tcp, _) = listener.accept()?;
    tcp.set_nonblocking(true)?;
    info!("TCP connection accepted");

    let mut buf = [0u8; 2];
    let mut tcp_buf = [0u8; 1];
    let mut latest_keys: Option<u16> = None;

    while let Some(event) = events.recv() {
        match event {
            Event::Input {
                id,
                metadata,
                data: _data,
            } => match id.as_str() {
                "tick" => {
                    trace!("tick input received");
                    while let Ok(n) = tcp.read(&mut tcp_buf)
                        && n > 0
                    {
                        match tcp_buf[0] {
                            1 => {
                                info!("start received");
                            }
                            2 => {
                                info!("shutdown received, exiting");
                                node.send_output(
                                    out_shutdown.clone(),
                                    metadata.parameters.clone(),
                                    true.into_arrow(),
                                )?;
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
                        let status = match keys {
                            0 => StatusController {
                                move_power: 0.0,
                                angle_horizontal: 0.0,
                                angle_vertical: 0.0,
                                roll_power: 0.0,
                            },
                            1 => StatusController {
                                move_power: 1.0,
                                angle_horizontal: 10.0,
                                angle_vertical: 5.0,
                                roll_power: 0.5,
                            },
                            2 => StatusController {
                                move_power: -1.0,
                                angle_horizontal: -10.0,
                                angle_vertical: -5.0,
                                roll_power: -0.5,
                            },
                            _ => StatusController {
                                move_power: 0.0,
                                angle_horizontal: 0.0,
                                angle_vertical: 0.0,
                                roll_power: 0.0,
                            },
                        };
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

    Ok(())
}
