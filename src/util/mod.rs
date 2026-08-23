use crate::model::ServerConfig;
use anyhow::Result;
use std::fs;

pub fn load_config(path: &str) -> Result<ServerConfig> {
    let data = fs::read_to_string(path)?;
    let cfg: ServerConfig = serde_json::from_str(&data)?;
    Ok(cfg)
}

pub fn init_logging() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();
}
