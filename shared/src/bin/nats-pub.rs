use anyhow::{Result, anyhow};
use clap::Parser;
use shared::{
    cli::{CommonArgs, merged_config},
    logging,
};
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "nats-pub")]
struct Args {
    #[command(flatten)]
    common: CommonArgs,

    /// Message body (use --msg "text") or provide as positional
    #[arg(long)]
    msg: Option<String>,

    /// Positional message (fallback)
    message: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    logging::init();
    let args = Args::parse();

    // Default lab config path, relative to the crate
    let default_lab_cfg = format!("{}/config.toml", env!("CARGO_MANIFEST_DIR"));
    let cfg = merged_config(&args.common, &default_lab_cfg)?;
    let url = cfg.nats_url()?;

    // Resolve subject: CLI > CommonArgs > config
    let subject = args
        .common
        .subject
        .clone()
        .or(cfg.nats.subject)
        .ok_or_else(|| {
            anyhow!("Subject is required. Pass --subject or set [nats].subject in config.")
        })?;

    // Resolve message: --msg > positional > default
    let body = args.msg.or(args.message).unwrap_or_else(|| "hello".into());

    info!(%url, %subject, body = %body, "publishing message");

    let client = async_nats::connect(&url).await?;
    client.publish(subject.clone(), body.into()).await?;
    client.flush().await?;

    info!(%subject, "message published");
    Ok(())
}
