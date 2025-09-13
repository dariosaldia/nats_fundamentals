use anyhow::{anyhow, Result};
use clap::Parser;
use shared::{
    cli::{merged_config, CommonArgs},
    logging,
};
use tokio::time::{timeout, Duration};
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "requester")]
struct Args {
    #[command(flatten)]
    common: CommonArgs,

    /// Message body (use --msg "text") or provide as positional
    #[arg(long)]
    msg: Option<String>,

    /// Positional message (fallback)
    message: Option<String>,

    /// Timeout to wait for the reply (milliseconds)
    #[arg(long, default_value_t = 2000)]
    timeout_ms: u64,

    /// Optional label for logs
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

    let body = args.msg.or(args.message).unwrap_or_else(|| "ping".into());
    let tmo = Duration::from_millis(args.timeout_ms);
    let label = args
        .label
        .clone()
        .unwrap_or_else(|| format!("requester-{}", std::process::id()));

    info!(%url, %subject, %label, req=%body, timeout_ms=%args.timeout_ms, "sending request");

    let client = async_nats::connect(&url).await?;

    // Send request and await reply with timeout
    let fut = client.request(subject.clone(), body.clone().into());
    let resp_msg = timeout(tmo, fut).await.map_err(|_| {
        anyhow!(
            "request timed out after {} ms to subject={}",
            args.timeout_ms,
            subject
        )
    })??;

    let resp = String::from_utf8_lossy(&resp_msg.payload);
    info!(%subject, %label, reply=%resp, "received reply");

    Ok(())
}