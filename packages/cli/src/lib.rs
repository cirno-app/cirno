use std::path::Path;

use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use tokio::fs::read_to_string;
use tokio::try_join;
use uuid::Uuid;

use crate::yarn::{YarnLock, YarnRc};

pub mod yarn;

pub const VERSION: &str = "1.0";

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
