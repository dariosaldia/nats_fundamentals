use anyhow::Result;
use clap::Args as ClapArgs;

use crate::config::AppConfig;

#[derive(Clone, Debug, ClapArgs)]
pub struct CommonArgs {
    /// Path to the root config TOML (required). Defaults to ./config.toml
    #[arg(long, default_value = "config.toml")]
    pub config: String,

    /// Optional path to the lab-scoped config TOML (merged over root)
    #[arg(long)]
    pub lab_config: Option<String>,

    /// Optional subject override from CLI
    #[arg(long)]
    pub subject: Option<String>,
}

pub fn merged_config(common: &CommonArgs, default_lab_cfg: &str) -> Result<AppConfig> {
    AppConfig::load_merged(
        &common.config,
        if common.lab_config.is_some() {
            common.lab_config.as_deref()
        } else if !default_lab_cfg.is_empty() {
            Some(default_lab_cfg)
        } else {
            None
        },
    )
}
