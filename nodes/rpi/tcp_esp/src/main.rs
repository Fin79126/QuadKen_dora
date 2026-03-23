// use dora_node_api::{self, DoraNode, Event};
use dora_node_api::{
    self, DoraNode, Event, EventStream, IntoArrow, dora_core::config::DataId, init_tracing,
};
use eyre::Context;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use tracing::{Level, debug, error, info, span};
use types::{MainCommand, Status};

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
    dotenv::dotenv().ok();

    let esp_ip = std::env::var("ESP_IP").unwrap_or_else(|_| "192.168.4.1".into());
    let esp_port = std::env::var("ESP_TCP_PORT").unwrap_or_else(|_| "5001".into());
    let addr = format!("{}:{}", esp_ip, esp_port);
    let out_status = DataId::from("status".to_owned());

    let span = span!(Level::INFO, "TX_ESP32_TCP");
    let _enter = span.enter();

    // =========================
    // TCP接続（ESP32へ）
    // =========================
    let mut stream = TcpStream::connect(&addr)?;
    stream.set_nodelay(true)?;

    let mut stream_clone = stream.try_clone()?;
    {
        thread::spawn(move || -> eyre::Result<()> {
            let mut buf = Vec::with_capacity(256);
            let mut len_buf = [0u8; 4];
            loop {
                buf.clear();

                stream_clone.read_exact(&mut len_buf)?;

                let len = u32::from_be_bytes(len_buf) as usize;
                // 2. 本体読む
                buf.resize(len, 0);
                stream_clone.read_exact(&mut buf)?;
                // 3. deserialize
                let status: Status = postcard::from_bytes(&buf)?;
                debug!("Received status: {:?}", status);

                // 👇 他ノードへ出力（Dora想定）
                // ※ API名は環境に合わせて調整してください
                node.send_output(out_status.clone(), Default::default(), status.into_arrow())?;
            }
        });
    }
    while let Some(event) = events.recv() {
        match event {
            Event::Input { id, data, .. } => match id.as_str() {
                "main_command" => {
                    let command = MainCommand::try_from(&data)?;
                    debug!("Received main command: {:?}", command);

                    let serialized = postcard::to_allocvec(&command)?;

                    // TCP送信
                    let len = (serialized.len() as u32).to_be_bytes();

                    stream.write_all(&len)?;
                    stream.write_all(&serialized)?;
                    stream.flush()?;
                }
                other => error!("Ignoring unexpected input `{other}`"),
            },

            Event::Stop(_) => {
                info!("Received stop");
                break;
            }

            other => error!("Unexpected event: {other:?}"),
        }
    }

    Ok(())
}
