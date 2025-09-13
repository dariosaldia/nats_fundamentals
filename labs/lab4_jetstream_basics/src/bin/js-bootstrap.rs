use anyhow::{Result, anyhow};
use async_nats::jetstream::{consumer, stream};
use clap::Parser;
use shared::{
    cli::{CommonArgs, merged_config},
    logging,
};
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "js-bootstrap")]
struct Args {
    #[command(flatten)]
    common: CommonArgs,

    /// Override stream name (else use [nats].stream)
    #[arg(long)]
    stream: Option<String>,

    /// Override consumer name (else use [nats].consumer)
    #[arg(long)]
    consumer: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    logging::init();
    let args = Args::parse();

    // Use this lab's config.toml by default
    let default_lab_cfg = format!("{}/config.toml", env!("CARGO_MANIFEST_DIR"));
    let cfg = merged_config(&args.common, &default_lab_cfg)?;
    let url = cfg.nats_url()?;

    let subject = cfg
        .nats
        .subject
        .ok_or_else(|| anyhow!("Missing [nats].subject (or --subject)"))?;

    let stream_name = args
        .stream
        .or(cfg.nats.stream.clone())
        .ok_or_else(|| anyhow!("Missing [nats].stream (or --stream)"))?;

    let consumer_name = args
        .consumer
        .or(cfg.nats.consumer.clone())
        .ok_or_else(|| anyhow!("Missing [nats].consumer (or --consumer)"))?;

    info!(%url, %stream_name, %subject, %consumer_name, "jetstream bootstrap starting");

    let client = async_nats::connect(&url).await?;
    let js = async_nats::jetstream::new(client);

    // Create or update the stream
    js.create_or_update_stream(stream::Config {
        name: stream_name.clone(),
        subjects: vec![subject.clone()],
        ..Default::default()
    })
    .await?;

    // Create a durable, pull-based consumer with explicit acks
    let stream = js.get_stream(stream_name.clone()).await?;
    let _ = stream
        .create_consumer(consumer::Config {
            durable_name: Some(consumer_name.clone()),
            ack_policy: consumer::AckPolicy::Explicit,
            deliver_policy: consumer::DeliverPolicy::All, // ensure stored msgs are delivered
            ..Default::default()
        })
        .await?;

    info!("bootstrap completed");
    Ok(())
}
