use std::collections::HashMap;
use std::sync::LazyLock;

use anyhow::{Result, anyhow};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct YarnRc {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_folder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_global_cache: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_tips: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_linker: Option<NodeLinker>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npm_registry_server: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pnp_enable_esm_loader: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yarn_path: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
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

/// Unique hash of a package descriptor. Used as key in various places so that
/// two descriptors can be quickly compared.
type IdentHash = String;

/// Combination of a scope and name, bound with a hash suitable for comparisons.
///
/// Use `parse_ident` to turn ident strings (`@types/node`) into the ident
/// structure `{scope: "types", name: "node"}`, `make_ident` to create a new one
/// from known parameters, or `stringify_ident` to retrieve the string as you'd
/// see it in the `dependencies` field.
pub struct Ident {
    /// Unique hash of a package scope and name. Used as key in various places,
    /// so that two idents can be quickly compared.
    pub ident_hash: IdentHash,
    /// Scope of the package, without the `@` prefix (eg. `types`).
    pub scope: Option<String>,
    /// Name of the package (eg. `node`).
    pub name: String,
}

impl Ident {
    pub fn new(scope: Option<String>, name: String) -> Self {
        let mut hasher = Sha512::new();
        if let Some(scope) = &scope {
            hasher.update(scope);
        }
        hasher.update(&name);
        Self {
            ident_hash: hex::encode(hasher.finalize()),
            scope,
            name,
        }
    }

    pub fn slugify(&self) -> String {
        if let Some(scope) = &self.scope {
            format!("@{}-{}", scope, self.name)
        } else {
            self.name.clone()
        }
    }
}

/// Unique hash of a package descriptor. Used as key in various places so that
/// two descriptors can be quickly compared.
type DescriptorHash = String;

/// Descriptors are just like idents (including their `identHash`), except that
/// they also contain a range and an additional comparator hash.
///
/// Use `parseRange` to turn a descriptor string into this data structure,
/// `makeDescriptor` to create a new one from an ident and a range, or
/// `stringifyDescriptor` to generate a string representation of it.
pub struct Descriptor {
    pub ident: Ident,
    /// Unique hash of a package descriptor. Used as key in various places, so
    /// that two descriptors can be quickly compared.
    pub descriptor_hash: DescriptorHash,
    /// The range associated with this descriptor. (eg. `^1.0.0`)
    pub range: String,
}

static LOCATOR_REGEX_STRICT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(?:@([^/]+?)\/)?([^@/]+?)(?:@(.+))$").unwrap());
static LOCATOR_REGEX_LOOSE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(?:@([^/]+?)\/)?([^@/]+?)(?:@(.+))?$").unwrap());

/// Unique hash of a package locator. Used as key in various places so that
/// two locators can be quickly compared.
type LocatorHash = String;

/// Locator are just like idents (including their `identHash`), except that
/// they also contain a reference and an additional comparator hash. They are
/// in this regard very similar to descriptors except that each descriptor may
/// reference multiple valid candidate packages whereas each locators can only
/// reference a single package.
///
/// This interesting property means that each locator can be safely turned into
/// a descriptor (using `convertLocatorToDescriptor`), but not the other way
/// around (except in very specific cases).
pub struct Locator {
    pub ident: Ident,
    /// Unique hash of a package locator. Used as key in various places so that
    /// two locators can be quickly compared.
    pub locator_hash: LocatorHash,
    /// A package reference uniquely identifies a package (eg. `1.2.3`).
    pub reference: String,
}

impl Locator {
    pub fn new(ident: Ident, reference: String) -> Self {
        let mut hasher = Sha512::new();
        hasher.update(&ident.ident_hash);
        hasher.update(&reference);
        Self {
            ident,
            locator_hash: hex::encode(hasher.finalize()),
            reference,
        }
    }

    pub fn slugify(&self) -> String {
        todo!()
    }

    pub fn try_parse(string: &str, strict: bool) -> Option<Self> {
        let regex = if strict { &*LOCATOR_REGEX_STRICT } else { &*LOCATOR_REGEX_LOOSE };
        let captures = regex.captures(string)?;
        let scope = captures.get(1).map(|m| m.as_str().to_string());
        let name = captures[2].to_string();
        let reference = captures
            .get(3)
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let ident = Ident::new(scope, name);
        Some(Locator::new(ident, reference))
    }
}

impl YarnLock {
    pub fn get_cache_files(&self) -> Result<Vec<String>> {
        self.packages.values().try_fold(Vec::new(), |mut acc, value| {
            let locator = Locator::try_parse(&value.resolution, true)
                .ok_or_else(|| anyhow!("Failed to parse resolution: {}", value.resolution))?;
            if !locator.reference.starts_with("workspace:") {
                acc.push(locator.slugify());
            }
            Ok(acc)
        })
    }
}
