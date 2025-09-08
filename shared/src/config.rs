use anyhow::{Context, Result, anyhow};
use config::{Config, Environment, File};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RuntimeConfig {
    pub mode: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct NatsConfig {
    pub url: Option<String>,
    pub subject: Option<String>,
    pub queue: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RecvConfig {
    pub wait_secs: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub runtime: RuntimeConfig,
    #[serde(default)]
    pub nats: NatsConfig,
    #[serde(default)]
    pub recv: RecvConfig,
}

impl AppConfig {
    /// Load and MERGE:
    ///  - root_config (e.g., config.toml at repo root)
    ///  - lab_config  (e.g., labs/<lab>/config.toml)  — later source overrides earlier
    ///  - environment (APP_* with "__" nesting)       — highest precedence
    pub fn load_merged(root_config: &str, lab_config: Option<&str>) -> Result<Self> {
        let mut builder = Config::builder();

        // Root config (required)
        if !Path::new(root_config).exists() {
            return Err(anyhow!(
                "Root config not found at '{}'. Create it (e.g. copy config.example.toml) or pass --config <path>.",
                root_config
            ));
        }
        builder = builder.add_source(File::with_name(root_config));

        // Lab config (optional, overrides root)
        if let Some(lab_path) = lab_config {
            if Path::new(lab_path).exists() {
                builder = builder.add_source(File::with_name(lab_path));
            }
        }

        // Environment overrides (e.g., APP_NATS__URL=nats://localhost:4222)
        builder = builder.add_source(
            Environment::with_prefix("APP")
                .separator("__")
                .try_parsing(true),
        );

        let cfg = builder.build().context("building merged config")?;
        let mut out: AppConfig = cfg.try_deserialize().context("deserializing AppConfig")?;

        // Defaults
        if out.recv.wait_secs.is_none() {
            out.recv.wait_secs = Some(5);
        }

        Ok(out)
    }

    pub fn nats_url(&self) -> Result<String> {
        self.nats
            .url
            .clone()
            .ok_or_else(|| anyhow!("Missing [nats].url in config"))
    }

    pub fn subject(&self) -> Option<String> {
        self.nats.subject.clone()
    }

    pub fn recv_wait_secs(&self) -> u64 {
        self.recv.wait_secs.unwrap_or(5)
    }
}
