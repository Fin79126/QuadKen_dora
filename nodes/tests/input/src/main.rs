// use dora_node_api::{self, DoraNode, Event};
use dora_node_api::{self, DoraNode, Event, EventStream, init_tracing};
use eyre::Context;
use tracing::{Level, error, info, span};

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
                    "input_test" => {
                        info!("Raw data: {:?}", data);
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
