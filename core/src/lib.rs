use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use brotli::BrotliCompress;
use serde::{Deserialize, Serialize};
use tokio::fs::{create_dir, create_dir_all, read_to_string, write};
use uuid::Uuid;

use crate::yarn::{NodeLinker, YarnLock, YarnRc};

pub mod yarn;

// const VERSION: &str = "1.0";
const ENTRY_FILE: &str = "cirno.yml";
const STATE_FILE: &str = "cirno-baka.br";

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
        let package = serde_json::from_str(&read_to_string(&cwd.join("package.json")).await?)?;
        let yarn_rc = serde_yaml_ng::from_str(&read_to_string(&cwd.join(".yarnrc.yml")).await?)?;
        let yarn_lock = serde_json::from_str(&read_to_string(&cwd.join("yarn.lock")).await?)?;
        Ok(Meta {
            package,
            yarn_rc,
            yarn_lock,
        })
    }
}

pub struct Cirno {
    cwd: PathBuf,
    data: Manifest,
    apps: HashMap<Uuid, App>,
}

impl Cirno {
    pub async fn init(&self) -> Result<()> {
        create_dir_all(&self.cwd).await.context("Failed to create cirno dir")?;
        create_dir(&self.cwd.join("apps"))
            .await
            .context("Failed to create apps dir")?;
        create_dir(&self.cwd.join("baka"))
            .await
            .context("Failed to create baka dir")?;
        create_dir(&self.cwd.join("home"))
            .await
            .context("Failed to create home dir")?;
        create_dir(&self.cwd.join("home/.yarn"))
            .await
            .context("Failed to create yarn dir")?;
        create_dir(&self.cwd.join("home/.yarn/cache"))
            .await
            .context("Failed to create yarn cache dir")?;
        create_dir(&self.cwd.join("home/.yarn/releases"))
            .await
            .context("Failed to create yarn releases dir")?;
        let tmp_dir = self.cwd.join("tmp");
        create_dir(&tmp_dir).await.context("Failed to create tmp dir")?;
        #[cfg(target_os = "windows")]
        {
            create_dir_all(&self.cwd.join("home/AppData"))
                .await
                .context("Failed to create AppData dir")?;
            create_dir_all(&self.cwd.join("home/AppData/Local"))
                .await
                .context("Failed to create AppData Local dir")?;
            create_dir_all(&self.cwd.join("home/AppData/Roaming"))
                .await
                .context("Failed to create AppData Roaming dir")?;
        }
        let yarn_rc = YarnRc {
            enable_tips: Some(false),
            node_linker: Some(NodeLinker::Pnp),
            pnp_enable_esm_loader: Some(true),
            ..YarnRc::default()
        };
        write(&self.cwd.join("home/.yarnrc.yml"), &serde_yaml_ng::to_string(&yarn_rc)?)
            .await
            .context("Failed to write default .yarnrc.yml")?;
        Ok(())
    }

    pub async fn save(&self) -> Result<()> {
        // this.data.apps = Object.entries(this.apps)
        //     .filter(([id, app]) => id === app.id)
        //     .map(([_, app]) => app)
        write(&self.cwd.join(ENTRY_FILE), &serde_yaml_ng::to_string(&self.data)?)
            .await
            .context("Failed to write cirno.yml")?;
        let str = serde_json::to_string(&self.apps)?;
        let mut output = Vec::new();
        BrotliCompress(&mut str.as_bytes(), &mut output, &Default::default())?;
        write(&self.cwd.join(STATE_FILE), output)
            .await
            .context("Failed to write cirno-baka.br")?;
        Ok(())
    }
}
