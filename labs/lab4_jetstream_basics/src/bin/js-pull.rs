use anyhow::{Result, anyhow};
use async_nats::jetstream::consumer;
use clap::Parser;
use futures::StreamExt;
use shared::{
    cli::{CommonArgs, merged_config},
    logging,
};
use tracing::{info, warn};

#[derive(Parser, Debug)]
#[command(name = "js-pull")]
struct Args {
    #[command(flatten)]
    common: CommonArgs,

    /// Override stream name (else use [nats].stream)
    #[arg(long)]
    stream: Option<String>,

    /// Override consumer name (else use [nats].consumer)
    #[arg(long)]
    consumer: Option<String>,

    /// Max messages to print before exiting (0 = infinite)
    #[arg(long, default_value_t = 0)]
    max: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    logging::init();
    let args = Args::parse();

    let default_lab_cfg = format!("{}/config.toml", env!("CARGO_MANIFEST_DIR"));
    let cfg = merged_config(&args.common, &default_lab_cfg)?;
    let url = cfg.nats_url()?;

    let stream_name = args
        .stream
        .or(cfg.nats.stream.clone())
        .ok_or_else(|| anyhow!("Missing [nats].stream (or --stream)"))?;

    let consumer_name = args
        .consumer
        .or(cfg.nats.consumer.clone())
        .ok_or_else(|| anyhow!("Missing [nats].consumer (or --consumer)"))?;

    info!(%url, %stream_name, %consumer_name, "pull consumer starting");

    let client = async_nats::connect(&url).await?;
    let js = async_nats::jetstream::new(client);

    let stream = js
        .get_stream(stream_name)
        .await
        .map_err(|e| anyhow!("failed to get stream: {e}"))?;

    let consumer: consumer::PullConsumer = stream
        .get_consumer(&consumer_name)
        .await
        .map_err(|e| anyhow!("failed to get consumer: {e}"))?;

    let mut messages = consumer
        .messages()
        .await
        .map_err(|e| anyhow!("failed to create consumer message stream: {e}"))?;
    let mut seen = 0u64;

    info!("waiting for messages (Ctrl+C to stop, or --max N)");

    while let Some(res) = messages.next().await {
        match res {
            Ok(msg) => {
                let body = String::from_utf8_lossy(&msg.payload);
                info!(%body, "pulled");
                msg.ack().await.map_err(|e| anyhow!("ack failed: {e}"))?;
                info!("acked");
                seen += 1;
                if args.max > 0 && seen >= args.max {
                    break;
                }
            }
            Err(e) => {
                warn!(error=%e, "message fetch error");
                break; // or continue
            }
        }
    }

    Ok(())
}
