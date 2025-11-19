use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct YarnRc {
    #[serde(skip_serializing_if = "Option::is_none")]
    cache_folder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    enable_global_cache: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    node_linker: Option<NodeLinker>,
    #[serde(skip_serializing_if = "Option::is_none")]
    npm_registry_server: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    yarn_path: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NodeLinker {
    NodeModules,
    Pnp,
    Pnpm,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YarnLock {
    #[serde(rename = "__metadata")]
    pub metadata: YarnLockMetadata,
    #[serde(flatten)]
    pub packages: HashMap<String, YarnLockEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YarnLockMetadata {
    version: u32,
    cache_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YarnLockEntry {
    version: String,
    resolution: String,
    dependencies: Option<HashMap<String, String>>,
    checksum: String,
    language_name: String,
    link_type: LinkType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LinkType {
    Hard,
    Soft,
}
