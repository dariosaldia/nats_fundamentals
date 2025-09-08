use anyhow::{Result, anyhow};
use clap::Parser;
use futures::StreamExt;
use shared::{
    cli::{CommonArgs, merged_config},
    logging,
};
use tracing::{info, warn};

#[derive(Parser, Debug)]
#[command(name = "nats-sub")]
struct Args {
    #[command(flatten)]
    common: CommonArgs,
}

#[tokio::main]
async fn main() -> Result<()> {
    logging::init();
    let args = Args::parse();

    // Infer default lab config path from the lab crate running this binary.
    // (If running directly from shared, pass --lab-config explicitly.)
    let default_lab_cfg = format!("{}/config.toml", env!("CARGO_MANIFEST_DIR"));
    let cfg = merged_config(&args.common, &default_lab_cfg)?;
    let url = cfg.nats_url()?;

    // Resolve subject: CLI > CommonArgs > config
    let subject = args
        .common
        .subject
        .clone()
        .or(cfg.subject())
        .ok_or_else(|| {
            anyhow!("Subject is required. Pass --subject or set [nats].subject in config.")
        })?;

    info!(%url, %subject, "subscriber starting");

    let client = async_nats::connect(url).await?;
    let mut sub = client.subscribe(subject.clone()).await?;

    info!("waiting for messages (Ctrl+C to stop)");

    while let Some(msg) = sub.next().await {
        let body = String::from_utf8_lossy(&msg.payload);
        info!(%subject, body = %body, "message received");
        if msg.payload.is_empty() {
            warn!(%subject, "received empty payload");
        }
    }

    Ok(())
}
