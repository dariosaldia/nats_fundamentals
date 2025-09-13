use anyhow::{anyhow, Result};
use clap::Parser;
use futures::StreamExt;
use shared::{
    cli::{merged_config, CommonArgs},
    logging,
};
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "responder")]
struct Args {
    #[command(flatten)]
    common: CommonArgs,

    /// Optional label to identify this responder instance
    #[arg(long)]
    label: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    logging::init();
    let args = Args::parse();

    // Use the lab's config.toml by default
    let default_lab_cfg = format!("{}/config.toml", env!("CARGO_MANIFEST_DIR"));
    let cfg = merged_config(&args.common, &default_lab_cfg)?;
    let url = cfg.nats_url()?;

    // Resolve subject: CLI > CommonArgs > config
    let subject = args
        .common
        .subject
        .clone()
        .or(cfg.nats.subject)
        .ok_or_else(|| anyhow!("Subject is required. Use --subject or set [nats].subject"))?;

    let label = args
        .label
        .clone()
        .unwrap_or_else(|| format!("responder-{}", std::process::id()));

    info!(%url, %subject, %label, "responder starting");

    let client = async_nats::connect(&url).await?;
    let mut sub = client.subscribe(subject.clone()).await?;

    info!("listening for requests (Ctrl+C to stop)");

    while let Some(msg) = sub.next().await {
        let body = String::from_utf8_lossy(&msg.payload);
        info!(%label, req=%body, "received request");

        let reply = format!("ack: {}", body);
        // If a reply subject exists, respond; otherwise just log.
        if let Some(reply_to) = msg.reply {
            client.publish(reply_to, reply.clone().into()).await?;
            // Ensure the reply is flushed to the server before looping
            client.flush().await?;
            info!(%label, resp=%reply, "sent reply");
        } else {
            info!(%label, "request had no reply subject; nothing to send");
        }
    }

    Ok(())
}