use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;
use std::sync::LazyLock;

use anyhow::{Result, anyhow};
use brotli::{BrotliCompress, BrotliDecompress};
use futures::future::try_join_all;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::process::Command;
use uuid::Uuid;

use crate::yarn::{NodeLinker, YarnLock, YarnRc};

pub mod fs;
pub mod yarn;

const VERSION: &str = "1.0";
const ENTRY_FILE: &str = "cirno.yml";
const STATE_FILE: &str = "cirno-baka.br";

static YARN_CACHE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(.+)-([0-9a-f]+)\.zip$").unwrap());
static PACKAGE_MANAGER_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^([\w-]+)@(\d+\.\d+\.\d+)$").unwrap());

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

pub enum InitError {
    NotEmpty,
    Other(anyhow::Error),
}

impl<T: Into<anyhow::Error>> From<T> for InitError {
    fn from(value: T) -> Self {
        Self::Other(value.into())
    }
}

pub enum OpenError {
    Empty,
    Version(String),
    Other(anyhow::Error),
}

impl<T: Into<anyhow::Error>> From<T> for OpenError {
    fn from(value: T) -> Self {
        Self::Other(value.into())
    }
}

fn normalize_path(path: &Path) -> Result<PathBuf, std::io::Error> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

async fn get_file_count(cwd: &Path) -> Result<usize, std::io::Error> {
    let mut len = 0;
    let mut dir = tokio::fs::read_dir(cwd).await?;
    while dir.next_entry().await?.is_some() {
        len += 1;
    }
    Ok(len)
}

pub struct Cirno {
    cwd: PathBuf,
    data: Manifest,
    apps: HashMap<String, App>,
    state: HashMap<String, HashMap<String, Meta>>,
}

impl Cirno {
    pub async fn init(cwd: &Path, force: bool) -> Result<PathBuf, InitError> {
        let cwd = normalize_path(cwd)?;
        match get_file_count(&cwd).await {
            Ok(0) => {}
            Ok(_) => {
                if !force {
                    return Err(InitError::NotEmpty);
                }
                fs::remove_dir_all(&cwd).await?;
            }
            Err(error) => match error.kind() {
                ErrorKind::NotFound => {}
                _ => return Err(error.into()),
            },
        }
        fs::create_dir_all(&cwd).await?;
        fs::create_dir(cwd.join("apps")).await?;
        fs::create_dir(cwd.join("baka")).await?;
        fs::create_dir(cwd.join("home")).await?;
        fs::create_dir(cwd.join("home/.yarn")).await?;
        fs::create_dir(cwd.join("home/.yarn/cache")).await?;
        fs::create_dir(cwd.join("home/.yarn/releases")).await?;
        fs::create_dir(cwd.join("tmp")).await?;
        #[cfg(target_os = "windows")]
        {
            fs::create_dir(cwd.join("home/AppData")).await?;
            fs::create_dir(cwd.join("home/AppData/Local")).await?;
            fs::create_dir(cwd.join("home/AppData/Roaming")).await?;
        }
        let yarn_rc = YarnRc {
            enable_tips: Some(false),
            enable_telemetry: Some(false),
            node_linker: Some(NodeLinker::Pnp),
            pnp_enable_esm_loader: Some(true),
            ..YarnRc::default()
        };
        fs::write(cwd.join("home/.yarnrc.yml"), &serde_yaml_ng::to_string(&yarn_rc)?).await?;
        let cirno = Self {
            cwd,
            data: Manifest {
                version: VERSION.to_string(),
                apps: vec![],
            },
            apps: Default::default(),
            state: Default::default(),
        };
        cirno.save().await?;
        Ok(cirno.cwd)
    }

    pub async fn open(cwd: &Path) -> Result<Self, OpenError> {
        let cwd = normalize_path(cwd)?;
        match get_file_count(&cwd).await {
            Ok(0) => return Err(OpenError::Empty),
            Err(error) => match error.kind() {
                ErrorKind::NotFound => return Err(OpenError::Empty),
                _ => return Err(error.into()),
            },
            _ => {}
        }
        let manifest: Manifest = serde_yaml_ng::from_str(&fs::read_to_string(cwd.join(ENTRY_FILE)).await?)?;
        if manifest.version != VERSION {
            return Err(OpenError::Version(manifest.version));
        }
        let mut output = vec![];
        BrotliDecompress(&mut fs::read(cwd.join(STATE_FILE)).await?.as_slice(), &mut output)?;
        let state: HashMap<String, HashMap<String, Meta>> = serde_json::from_str(std::str::from_utf8(&output)?)?;
        Ok(Self {
            cwd,
            data: manifest,
            apps: Default::default(), // TODO
            state,
        })
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

    pub async fn yarn<I, S>(&self, cwd: &Path, args: I) -> Result<ExitStatus>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let pkg_meta: Package = serde_json::from_str(&fs::read_to_string(&cwd.join("package.json")).await?)?;
        let Some(captures) = PACKAGE_MANAGER_REGEX.captures(&pkg_meta.package_manager) else {
            return Err(anyhow!("Invalid package manager: {}", pkg_meta.package_manager));
        };
        if captures[1] != *"yarn" {
            return Err(anyhow!("Unsupported package manager: {}", &captures[1]));
        }
        let yarn_path = self.cwd.join(format!("home/.yarn/releases/yarn-{}.cjs", &captures[2]));
        let mut command = Command::new("node");
        command
            .arg(&yarn_path)
            .args(args)
            .current_dir(cwd)
            .envs(std::env::vars_os())
            .env("HOME", self.cwd.join("home"))
            .env("TEMP", self.cwd.join("tmp"))
            .env("TMP", self.cwd.join("tmp"))
            .env("TMPDIR", self.cwd.join("tmp"))
            .env("YARN_YARN_PATH", &yarn_path)
            .env("YARN_GLOBAL_FOLDER", self.cwd.join("home/.yarn"));
        for key in ["HOME", "TEMP", "TMP", "TMPDIR"] {
            if let Some(value) = std::env::var_os(key) {
                command.env(format!("CIRNO_HOST_{}", key), value);
            }
        }
        #[cfg(target_os = "windows")]
        {
            command
                .env("APPDATA", self.cwd.join("home/AppData/Roaming"))
                .env("LOCALAPPDATA", self.cwd.join("home/AppData/Local"))
                .env("USERPROFILE", self.cwd.join("home"));
            for key in ["APPDATA", "LOCALAPPDATA", "USERPROFILE"] {
                if let Some(value) = std::env::var_os(key) {
                    command.env(format!("CIRNO_HOST_{}", key), value);
                }
            }
        }
        Ok(command.status().await?)
    }

    pub async fn load_cache(&self) -> Result<HashMap<String, HashMap<String, String>>> {
        let mut cache = HashMap::<String, HashMap<String, String>>::new();
        let mut dir = fs::read_dir(self.cwd.join("home/.yarn/cache")).await?;
        while let Some(entry) = dir.next_entry().await? {
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
        let mut dir = fs::read_dir(self.cwd.join("home/.yarn/releases")).await?;
        while let Some(entry) = dir.next_entry().await? {
            let name = entry.file_name().to_string_lossy().to_string();
            releases.insert(name);
        }
        for meta_map in self.state.values() {
            for meta in meta_map.values() {
                if let Some(captures) = PACKAGE_MANAGER_REGEX.captures(&meta.package.package_manager)
                    && captures[1] == *"yarn"
                {
                    releases.remove(&format!("yarn-{}.cjs", &captures[2]));
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
