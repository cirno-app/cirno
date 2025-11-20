use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::Deserialize;
use tokio::fs::{create_dir_all, read_to_string};

pub struct EnvironmentState {
    pub exe_dir: PathBuf,
    pub bin_dir: PathBuf,
    pub config_path: PathBuf,

    pub data_dir: PathBuf,
    pub apps_dir: PathBuf,
    pub baka_dir: PathBuf,
    pub home_dir: PathBuf,
    pub home_yarn_dir: PathBuf,
    pub home_yarn_cache_dir: PathBuf,
    pub home_yarn_releases_dir: PathBuf,
    pub home_appdata_dir: PathBuf,
    pub home_appdata_local_dir: PathBuf,
    pub home_appdata_roaming_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub tmp_dir: PathBuf,

    pub config: CirnoConfig,
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
    // bin_dir does not apply config redirect
    let bin_dir = exe_dir.join("bin");

    let data_dir = exe_dir.join("data");

    // create_dir_all(data_dir.clone())
    //     .await
    //     .context("Failed to create data dir")?;

    let config_path = data_dir.join("cirno.yml");

    let config = serde_yaml::from_str(&read_to_string(config_path.clone()).await.context("Failed to read config file")?)
        .context("Failed to parse config file")?;

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

    create_dir_all(apps_dir.clone()).await.context("Failed to create apps dir")?;
    create_dir_all(baka_dir.clone()).await.context("Failed to create baka dir")?;
    // create_dir_all(home_dir.clone())
    //     .await
    //     .context("Failed to create home dir")?;
    // create_dir_all(home_yarn_dir.clone())
    //     .await
    //     .context("Failed to create yarn dir")?;
    create_dir_all(home_yarn_cache_dir.clone())
        .await
        .context("Failed to create yarn cache dir")?;
    create_dir_all(home_yarn_releases_dir.clone())
        .await
        .context("Failed to create yarn releases dir")?;
    #[cfg(target_os = "windows")]
    {
        // create_dir_all(home_appdata_dir.clone())
        //     .await
        //     .context("Failed to create AppData dir")?;
        create_dir_all(home_appdata_local_dir.clone())
            .await
            .context("Failed to create AppData Local dir")?;
        create_dir_all(home_appdata_roaming_dir.clone())
            .await
            .context("Failed to create AppData Roaming dir")?;
    }
    create_dir_all(logs_dir.clone()).await.context("Failed to create logs dir")?;
    create_dir_all(tmp_dir.clone()).await.context("Failed to create tmp dir")?;

    Ok(EnvironmentState {
        exe_dir,
        bin_dir,
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
