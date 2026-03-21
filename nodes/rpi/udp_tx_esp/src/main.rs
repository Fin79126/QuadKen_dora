// use dora_node_api::{self, DoraNode, Event};
use dora_node_api::{self, DoraNode, Event, EventStream, init_tracing};
use eyre::Context;
use std::net::UdpSocket;
use tracing::{Level, error, info, span};
use types::MotorCommand;

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

fn run(_: DoraNode, mut events: EventStream) -> eyre::Result<()> {
    dotenv::dotenv().ok(); // .envから環境変数を読み込む
    let esp_ip = std::env::var("ESP_IP").unwrap_or_else(|_| "192.168.4.1".into());
    let esp_port = std::env::var("ESP_PORT").unwrap_or_else(|_| "5001".into());
    let socket = UdpSocket::bind("0.0.0.0:0")?; // 適当なポートでOK
    let target_addr: &str = &format!("{}:{}", esp_ip, esp_port); // ESP32側a

    let span = span!(Level::INFO, "TX_ESP32");
    let _enter = span.enter();

    while let Some(event) = events.recv() {
        match event {
            Event::Input {
                id,
                metadata: _,
                data,
            } => match id.as_str() {
                "command" => {
                    let command = MotorCommand::try_from(&data)?;
                    info!("Received motor command: {:?}", command);

                    let serialized = postcard::to_vec::<MotorCommand, 96>(&command)?;

                    // 👇 UDP送信
                    socket.send_to(&serialized, target_addr)?;
                }
                other => error!("Ignoring unexpected input `{other}`"),
            },
            Event::Stop(_) => info!("Received stop"),
            other => error!("Received unexpected input: {other:?}"),
        }
    }

    Ok(())
}
