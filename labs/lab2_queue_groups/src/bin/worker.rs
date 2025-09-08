use anyhow::{Result, anyhow};
use clap::Parser;
use futures::StreamExt;
use shared::{
    cli::{CommonArgs, merged_config},
    logging,
};
use tracing::{info, warn};

#[derive(Parser, Debug)]
#[command(name = "worker")]
struct Args {
    #[command(flatten)]
    common: CommonArgs,

    /// Queue group name (all workers with the same name compete)
    #[arg(long)]
    queue: Option<String>,

    /// Optional worker label to print (defaults to hostname/pid if omitted)
    #[arg(long)]
    label: Option<String>,
}

fn default_label() -> String {
    let host = hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "unknown-host".to_string());
    format!("{}-{}", host, std::process::id())
}

#[tokio::main]
async fn main() -> Result<()> {
    logging::init();
    let args = Args::parse();

    // Use lab’s config.toml by default (this binary lives in the lab crate)
    let default_lab_cfg = format!("{}/config.toml", env!("CARGO_MANIFEST_DIR"));
    let cfg = merged_config(&args.common, &default_lab_cfg)?;
    let url = cfg.nats_url()?;
    let subject = args
        .common
        .subject
        .clone()
        .or(cfg.subject())
        .ok_or_else(|| anyhow!("Subject is required. Use --subject or set [nats].subject"))?;
    let label = args.label.clone().unwrap_or_else(default_label);

    let queue = args
        .queue
        .clone()
        .or(cfg.nats.queue.clone())
        .ok_or_else(|| anyhow!("Queue group is required. Use --queue or set [nats].queue"))?;

    info!(%url, %subject, %queue, %label, "worker starting (queue group)");

    let client = async_nats::connect(&url).await?;
    let mut sub = client
        .queue_subscribe(subject.clone(), queue.clone())
        .await?;

    info!("waiting for messages… (Ctrl+C to stop)");
    while let Some(msg) = sub.next().await {
        let body = String::from_utf8_lossy(&msg.payload);
        info!(%label, %subject, body=%body, "processed");
        // simulate quick processing (no ack concept in core NATS)
        if msg.payload.is_empty() {
            warn!(%label, "empty payload");
        }
    }

    Ok(())
}
