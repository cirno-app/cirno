use std::path::PathBuf;

use anyhow::Result;
use serde::Deserialize;
use tokio::fs::read_to_string;

pub struct EnvironmentState {
    exe_dir: PathBuf,
    data_dir: PathBuf,
    config_path: PathBuf,

    config: CirnoConfig,
}

#[derive(Deserialize, Debug)]
pub struct CirnoConfig {
    version: String,

    config: CirnoConfigConfig,

    apps: Vec<CirnoConfigApps>,
}

#[derive(Deserialize, Debug)]
pub struct CirnoConfigConfig {}

#[derive(Deserialize, Debug)]
pub struct CirnoConfigApps {}

pub async fn load_config(exe_dir: PathBuf) -> Result<EnvironmentState> {
    let data_dir = exe_dir.join("data");
    let config_path = data_dir.join("cirno.yml");

    let config = serde_yaml::from_str(&read_to_string(config_path.clone()).await?)?;

    Ok(EnvironmentState {
        exe_dir,
        data_dir,
        config_path,
        config,
    })
}
