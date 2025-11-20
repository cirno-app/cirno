use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use anyhow::Result;
use brotli::BrotliCompress;
use futures::future::try_join_all;
use regex::Regex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::yarn::{NodeLinker, YarnLock, YarnRc};

pub mod fs;
pub mod yarn;

// const VERSION: &str = "1.0";
const ENTRY_FILE: &str = "cirno.yml";
const STATE_FILE: &str = "cirno-baka.br";

static YARN_CACHE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(.+)-([0-9a-f]+)\.zip$").unwrap());
static PACKAGE_MANAGER_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^yarn@(\d+\.\d+\.\d+)$").unwrap());

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
        let package = serde_json::from_str(&fs::read_to_string(&cwd.join("package.json")).await?)?;
        let yarn_rc = serde_yaml_ng::from_str(&fs::read_to_string(&cwd.join(".yarnrc.yml")).await?)?;
        let yarn_lock = serde_json::from_str(&fs::read_to_string(&cwd.join("yarn.lock")).await?)?;
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
    state: HashMap<String, HashMap<String, Meta>>,
}

impl Cirno {
    pub async fn init(&self) -> Result<()> {
        fs::create_dir_all(&self.cwd).await?;
        fs::create_dir(&self.cwd.join("apps")).await?;
        fs::create_dir(&self.cwd.join("baka")).await?;
        fs::create_dir(&self.cwd.join("home")).await?;
        fs::create_dir(&self.cwd.join("home/.yarn")).await?;
        fs::create_dir(&self.cwd.join("home/.yarn/cache")).await?;
        fs::create_dir(&self.cwd.join("home/.yarn/releases")).await?;
        fs::create_dir(&self.cwd.join("tmp")).await?;
        #[cfg(target_os = "windows")]
        {
            fs::create_dir(&self.cwd.join("home/AppData")).await?;
            fs::create_dir(&self.cwd.join("home/AppData/Local")).await?;
            fs::create_dir(&self.cwd.join("home/AppData/Roaming")).await?;
        }
        let yarn_rc = YarnRc {
            enable_tips: Some(false),
            node_linker: Some(NodeLinker::Pnp),
            pnp_enable_esm_loader: Some(true),
            ..YarnRc::default()
        };
        fs::write(&self.cwd.join("home/.yarnrc.yml"), &serde_yaml_ng::to_string(&yarn_rc)?).await?;
        Ok(())
    }

    pub async fn save(&self) -> Result<()> {
        // this.data.apps = Object.entries(this.apps)
        //     .filter(([id, app]) => id === app.id)
        //     .map(([_, app]) => app)
        fs::write(&self.cwd.join(ENTRY_FILE), &serde_yaml_ng::to_string(&self.data)?).await?;
        let str = serde_json::to_string(&self.apps)?;
        let mut output = Vec::new();
        BrotliCompress(&mut str.as_bytes(), &mut output, &Default::default())?;
        fs::write(&self.cwd.join(STATE_FILE), output).await?;
        Ok(())
    }

    pub async fn clone(&self, app: &App, id: &Uuid, dest: &Path) -> Result<()> {
        if &app.id == id {
            fs::copy(self.cwd.join("apps").join(id.to_string()), dest).await?;
        } else {
            // const tar = new Tar(join(this.cwd, 'baka', id + '.tar.br'))
            // tar.load()
            // tar.extract(dest, 1)
            // await tar.finalize()
        }
        Ok(())
    }

    pub async fn load_cache(&self) -> Result<HashMap<String, HashMap<String, String>>> {
        let mut cache = HashMap::<String, HashMap<String, String>>::new();
        let mut entries = fs::read_dir(self.cwd.join("home/.yarn/cache")).await?;
        while let Some(entry) = entries.next_entry().await? {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let Some(captures) = YARN_CACHE_REGEX.captures(&name) else {
                continue;
            };
            cache
                .entry(captures[2].to_string())
                .or_default()
                .insert(captures[1].to_string(), name.to_string());
        }
        Ok(cache)
    }

    pub async fn gc(&self) -> Result<()> {
        let mut cache = self.load_cache().await?;
        let mut releases = HashSet::new();
        let mut entries = fs::read_dir(self.cwd.join("home/.yarn/releases")).await?;
        while let Some(entry) = entries.next_entry().await? {
            let name = entry.file_name().to_string_lossy().to_string();
            releases.insert(name);
        }
        for meta_map in self.state.values() {
            for meta in meta_map.values() {
                if let Some(captures) = PACKAGE_MANAGER_REGEX.captures(&meta.package.package_manager) {
                    releases.remove(&format!("yarn-{}.cjs", &captures[1]));
                }
                for locator in meta.yarn_lock.get_cache_files()? {
                    if let Some(locators) = cache.get_mut(&meta.yarn_lock.metadata.cache_key) {
                        locators.remove(&locator);
                    }
                }
            }
        }
        let release_dir = self.cwd.join("home/.yarn/releases");
        try_join_all(
            releases
                .into_iter()
                .map(async |name| fs::remove_file(&release_dir.join(&name)).await),
        )
        .await?;
        let cache_dir = self.cwd.join("home/.yarn/cache");
        try_join_all(
            cache
                .into_values()
                .flat_map(|locators| locators.into_values())
                .map(async |name| fs::remove_file(&cache_dir.join(&name)).await),
        )
        .await?;
        Ok(())
    }
}
