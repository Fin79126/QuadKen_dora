use dora_node_api::{
    self, DoraNode, Event, EventStream, IntoArrow, dora_core::config::DataId, init_tracing,
};
use eyre::Context;
use tracing::{Level, span};
use types::MainCommand;

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
    {
        let span = span!(Level::INFO, "Send Command Test");
        let _enter = span.enter();
        // use a fixed seed for reproducibility (we use this node's output in integration tests)
        fastrand::seed(42);
        let mut counter = 0;

        let output = DataId::from("main_command".to_owned());
        while let Some(event) = events.recv() {
            let event_span = span!(Level::DEBUG, "event");
            let _event_enter = event_span.enter();
            match event {
                Event::Input {
                    id,
                    metadata,
                    data: _,
                } => match id.as_str() {
                    "tick" => {
                        let process_span = span!(Level::INFO, "process_tick");
                        let _p = process_span.enter();

                        let command: MainCommand = match counter % 4 {
                            0 => MainCommand::AttachServo,
                            1 => MainCommand::DetachServo,
                            2 => MainCommand::GetStatus,
                            3 => MainCommand::Setup,
                            _ => unreachable!(),
                        };
                        counter += 1;

                        node.send_output(output.clone(), metadata.parameters, command.into_arrow())
                            .unwrap();
                    }
                    other => {
                        tracing::info!(input = other, "Ignoring unexpected input");
                    }
                },
                Event::Stop(_) => {
                    tracing::info!("Received stop event");
                }
                other => {
                    tracing::warn!(?other, "Received unexpected input");
                }
            }
        }
    }
    Ok(())
}
