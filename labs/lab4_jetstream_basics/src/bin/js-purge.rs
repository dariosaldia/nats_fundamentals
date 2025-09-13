use anyhow::{anyhow, Result};
use clap::Parser;
use shared::{
    cli::{merged_config, CommonArgs},
    logging,
};
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "js-purge")]
struct Args {
    #[command(flatten)]
    common: CommonArgs,

    /// Override stream name (else use [nats].stream)
    #[arg(long)]
    stream: Option<String>,
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

    info!(%url, %stream_name, "purging stream");

    let client = async_nats::connect(&url).await?;
    let js = async_nats::jetstream::new(client);

    let stream = js.get_stream(stream_name).await?;
    stream.purge().await?;

    info!("purged");
    Ok(())
}