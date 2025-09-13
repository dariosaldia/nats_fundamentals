use anyhow::{Result, anyhow};
use clap::Parser;
use shared::{
    cli::{CommonArgs, merged_config},
    logging,
};
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "js-pub")]
struct Args {
    #[command(flatten)]
    common: CommonArgs,

    /// Message body (required)
    #[arg(long, required = true)]
    msg: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    logging::init();
    let args = Args::parse();

    let default_lab_cfg = format!("{}/config.toml", env!("CARGO_MANIFEST_DIR"));
    let cfg = merged_config(&args.common, &default_lab_cfg)?;
    let url = cfg.nats_url()?;

    let subject = cfg
        .nats
        .subject
        .ok_or_else(|| anyhow!("Missing [nats].subject (or --subject)"))?;
    let body = args.msg;

    info!(%url, %subject, body=%body, "publishing to JetStream");

    let client = async_nats::connect(&url).await?;
    let js = async_nats::jetstream::new(client);

    // JetStream publish waits for a server ack (persisted)
    let ack_fut = js.publish(subject.clone(), body.into()).await?;
    ack_fut.await.map_err(|e| anyhow!("publish ack failed: {e}"))?;

    info!("published with server ack");
    Ok(())
}
