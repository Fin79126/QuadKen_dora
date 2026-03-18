use dora_node_api::{self, DoraNode, Event};
use tracing::{debug, error, info};

fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt()
        // .with_ansi(false)
        .with_env_filter("debug")
        .try_init()
        .ok(); // すでに初期化されている場合は無視

    // let out_shutdown = DataId::from("shutdown".to_owned());
    let (_, mut events) = DoraNode::init_from_env()?;

    while let Some(event) = events.recv() {
        match event {
            Event::Input {
                id,
                metadata: _,
                data,
            } => match id.as_str() {
                "input_test" => {
                    debug!("Raw data: {:?}", data);
                }
                other => error!("Ignoring unexpected input `{other}`"),
            },
            Event::Stop(_) => info!("Received stop"),
            other => error!("Received unexpected input: {other:?}"),
        }
    }

    Ok(())
}
