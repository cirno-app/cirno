use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Error, Result};
use serde::{Deserialize, Serialize};
use tokio::fs::{create_dir, create_dir_all, read_to_string, write};
use tokio::try_join;
use uuid::Uuid;

use crate::yarn::{YarnLock, YarnRc};

pub mod yarn;

// const VERSION: &str = "1.0";
// const ENTRY_FILE: &str = "cirno.yml";
// const STATE_FILE: &str = "cirno-baka.br";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub version: String,
    // pub config: Config,
    pub apps: Vec<App>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct App {
    pub id: Uuid,
    pub name: String,
    pub created: String, // TODO: time
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Backup {
    pub id: Uuid,
    pub r#type: Option<String>,
    pub message: Option<String>,
    pub created: String, // TODO: time
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Package {
    pub name: String,
    pub package_manager: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub package: Package,
    pub yarn_rc: YarnRc,
    pub yarn_lock: YarnLock,
}

impl Meta {
    pub async fn load(cwd: &Path) -> Result<Self> {
        let (package, yarn_rc, yarn_lock) = try_join!(
            async {
                let path = cwd.join("package.json");
                let content = read_to_string(&path).await?;
                Ok::<Package, Error>(serde_json::from_str(&content)?)
            },
            async {
                let path = cwd.join(".yarnrc.yml");
                let content = read_to_string(&path).await?;
                Ok::<YarnRc, Error>(serde_yaml_ng::from_str(&content)?)
            },
            async {
                let path = cwd.join("yarn.lock");
                let content = read_to_string(&path).await?;
                Ok::<YarnLock, Error>(serde_json::from_str(&content)?)
            },
        )?;

        Ok(Meta {
            package,
            yarn_rc,
            yarn_lock,
        })
    }
}

pub struct Cirno {
    cwd: PathBuf,
    apps: HashMap<Uuid, App>,
}

impl Cirno {
    pub async fn init(&self) -> Result<()> {
        create_dir_all(&self.cwd).await.context("Failed to create cirno dir")?;
        let apps_dir = self.cwd.join("apps");
        create_dir(&apps_dir).await.context("Failed to create apps dir")?;
        let baka_dir = self.cwd.join("baka");
        create_dir(&baka_dir).await.context("Failed to create baka dir")?;
        let home_dir = self.cwd.join("home");
        create_dir(&home_dir).await.context("Failed to create home dir")?;
        let home_yarn_dir = home_dir.join(".yarn");
        create_dir(&home_yarn_dir).await.context("Failed to create yarn dir")?;
        let home_yarn_cache_dir = home_yarn_dir.join("cache");
        create_dir(&home_yarn_cache_dir).await.context("Failed to create yarn cache dir")?;
        let home_yarn_releases_dir = home_yarn_dir.join("releases");
        create_dir(&home_yarn_releases_dir)
            .await
            .context("Failed to create yarn releases dir")?;
        let tmp_dir = self.cwd.join("tmp");
        create_dir(&tmp_dir).await.context("Failed to create tmp dir")?;
        #[cfg(target_os = "windows")]
        {
            let home_appdata_dir = home_dir.join("AppData");
            create_dir_all(&home_appdata_dir).await.context("Failed to create AppData dir")?;
            let home_appdata_local_dir = home_appdata_dir.join("Local");
            create_dir_all(&home_appdata_local_dir)
                .await
                .context("Failed to create AppData Local dir")?;
            let home_appdata_roaming_dir = home_appdata_dir.join("Roaming");
            create_dir_all(&home_appdata_roaming_dir)
                .await
                .context("Failed to create AppData Roaming dir")?;
        }
        let yarn_rc = YarnRc {
            enable_tips: Some(false),
            node_linker: Some(yarn::NodeLinker::Pnp),
            pnp_enable_esm_loader: Some(true),
            ..YarnRc::default()
        };
        write(&home_dir.join(".yarnrc.yml"), &serde_yaml_ng::to_string(&yarn_rc)?)
            .await
            .context("Failed to write default .yarnrc.yml")?;
        Ok(())
    }
}
