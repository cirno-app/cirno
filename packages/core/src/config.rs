use std::path::PathBuf;

use anyhow::Result;
use serde::Deserialize;
use tokio::fs::{create_dir, read_to_string};

pub struct EnvironmentState {
    exe_dir: PathBuf,
    config_path: PathBuf,

    data_dir: PathBuf,
    apps_dir: PathBuf,
    baka_dir: PathBuf,
    home_dir: PathBuf,
    home_yarn_dir: PathBuf,
    home_yarn_cache_dir: PathBuf,
    home_yarn_releases_dir: PathBuf,
    home_appdata_dir: PathBuf,
    home_appdata_local_dir: PathBuf,
    home_appdata_roaming_dir: PathBuf,
    logs_dir: PathBuf,
    tmp_dir: PathBuf,

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

    create_dir(data_dir.clone()).await?;

    let config_path = data_dir.join("cirno.yml");

    let config = serde_yaml::from_str(&read_to_string(config_path.clone()).await?)?;

    let apps_dir = data_dir.join("apps");
    let baka_dir = data_dir.join("baka");
    let home_dir = data_dir.join("home");
    let home_yarn_dir = home_dir.join(".yarn");
    let home_yarn_cache_dir = home_yarn_dir.join("cache");
    let home_yarn_releases_dir = home_yarn_dir.join("releases");
    let home_appdata_dir = home_dir.join("AppData");
    let home_appdata_local_dir = home_appdata_dir.join("Local");
    let home_appdata_roaming_dir = home_appdata_dir.join("Roaming");
    let logs_dir = data_dir.join("logs");
    let tmp_dir = data_dir.join("tmp");

    create_dir(apps_dir.clone()).await?;
    create_dir(baka_dir.clone()).await?;
    create_dir(home_dir.clone()).await?;
    create_dir(home_yarn_dir.clone()).await?;
    create_dir(home_yarn_cache_dir.clone()).await?;
    create_dir(home_yarn_releases_dir.clone()).await?;
    #[cfg(target_os = "windows")]
    {
        create_dir(home_appdata_dir.clone()).await?;
        create_dir(home_appdata_local_dir.clone()).await?;
        create_dir(home_appdata_roaming_dir.clone()).await?;
    }
    create_dir(logs_dir.clone()).await?;
    create_dir(tmp_dir.clone()).await?;

    Ok(EnvironmentState {
        exe_dir,
        config_path,

        data_dir,
        apps_dir,
        baka_dir,
        home_dir,
        home_yarn_dir,
        home_yarn_cache_dir,
        home_yarn_releases_dir,
        home_appdata_dir,
        home_appdata_local_dir,
        home_appdata_roaming_dir,
        logs_dir,
        tmp_dir,

        config,
    })
}
