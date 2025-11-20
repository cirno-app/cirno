use std::collections::HashMap;

use either::Either;
use serde::{Deserialize, Serialize};

/// Yarnrc files (named this way because they must be called `.yarnrc.yml`) are the one place where you'll be able to
/// configure Yarn's internal settings. While Yarn will automatically find them in the parent directories, they should
/// usually be kept at the root of your project (often your repository). Starting from the v2, they must be written in
/// valid Yaml and have the right extension (simply calling your file `.yarnrc` won't do).
///
/// Environment variables can be accessed from setting definitions by using the `${NAME}` syntax when defining the
/// values. By default Yarn will require the variables to be present, but this can be turned off by using either
/// `${NAME-fallback}` (which will return `fallback` if `NAME` isn't set) or `${NAME:-fallback}` (which will return
/// `fallback` if `NAME` isn't set, or is an empty string).
///
/// Finally, note that most settings can also be defined through environment variables (at least for the simpler ones;
/// arrays and objects aren't supported yet). To do this, just prefix the names and write them in snake case:
/// `YARN_CACHE_FOLDER` will set the cache folder (such values will overwrite any that might have been defined in the RC
/// files - use them sparingly).
///
/// See [https://yarnpkg.com/configuration/yarnrc].
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct YarnRc {
    /// Path where the downloaded packages are stored on your system.
    ///
    /// They'll be normalized, compressed, and saved under the form of zip archives with standardized names. The cache
    /// is deemed to be relatively safe to be shared by multiple projects, even when multiple Yarn instances run at the
    /// same time on different projects. For setting a global cache folder, you should use
    /// [`enable_global_cache`](Self::enable_global_cache) instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_folder: Option<String>,

    /// Behavior that Yarn should follow when it detects that a cache entry is outdated.
    ///
    /// Whether or not a cache entry is outdated depends on whether it has been built and checksumed by an earlier
    /// release of Yarn, or under a different compression settings. Possible behaviors are:
    ///
    /// - If [`RequiredOnly`](CacheMigrationMode::RequiredOnly), it'll keep using the file as-is, unless the version
    ///   that generated it was decidedly too old.
    /// - If [`MatchSpec`](CacheMigrationMode::MatchSpec), it'll also rebuild the file if the compression level has
    ///   changed.
    /// - If [`Always`](CacheMigrationMode::Always) (the default), it'll always regenerate the cache files so they use
    ///   the current cache version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_migration_mode: Option<CacheMigrationMode>,

    /// List of git refs against which Yarn will compare your branch when it needs to detect changes.
    ///
    /// Supports git branches, tags, and commits. The default configuration will compare against master, origin/master,
    /// upstream/master, main, origin/main, and upstream/main.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changeset_base_refs: Option<Vec<String>>,

    /// Array of file glob patterns that will be excluded from change detection.
    ///
    /// Files matching the following patterns (in terms of relative paths compared to the root of the project) will be
    /// ignored by every command checking whether files changed compared to the base ref (this include both `yarn
    /// version check` and `yarn workspaces foreach --since`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changeset_ignore_patterns: Option<Vec<String>>,

    /// Behavior that Yarn should follow when it detects that a cache entry has a different checksum than expected.
    ///
    /// Possible behaviors are:
    ///
    /// - If [`Throw`](ChecksumBehavior::Throw) (the default), Yarn will throw an exception.
    /// - If [`Update`](ChecksumBehavior::Update), the lockfile will be updated to match the cached checksum.
    /// - If [`Reset`](ChecksumBehavior::Reset), the cache entry will be purged and fetched anew.
    /// - If [`Ignore`](ChecksumBehavior::Ignore), nothing will happen, Yarn will skip the check.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum_behavior: Option<ChecksumBehavior>,

    /// Amount of `git clone` operations that Yarn will run at the same time.
    ///
    /// We by default limit it to 2 concurrent clone operations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clone_concurrency: Option<usize>,

    /// Compression level employed for zip archives
    ///
    /// Possible values go from [`L0`](CompressionLevel::L0) ("no compression, faster") to [`L9`](CompressionLevel::L9)
    /// ("heavy compression, slower"). The value [`Mixed`](CompressionLevel::Mixed) is a variant of
    /// [`L9`](CompressionLevel::L9) where files are stored uncompressed if the gzip overhead would exceed the size
    /// gain.
    ///
    /// The default is [`L0`](CompressionLevel::L0), which tends to be significantly faster to install. Projects using
    /// zero-installs are advised to keep it this way, as experiments showed that Git stores uncompressed package
    /// archives more efficiently than gzip-compressed ones.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression_level: Option<CompressionLevel>,

    /// Path of the constraints file.
    ///
    /// This only matters for Prolog constraints, which are being deprecated. JavaScript constraints will always be
    /// read from the `yarn.config.cjs` file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints_path: Option<String>,

    /// Default language mode that should be used when a package doesn't offer any insight.
    ///
    /// This is an internal configuration setting that shouldn't be touched unless you really know what you're doing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_language_name: Option<String>,

    /// Default protocol that should be used when a dependency range is a pure semver range.
    ///
    /// This is an internal configuration setting that shouldn't be touched unless you really know what you're doing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_protocol: Option<String>,

    /// Default prefix used in semver ranges created by `yarn add` and similar commands.
    ///
    /// Possible values are `"^"` (the default), `"~"` or `""`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_semver_range_prefix: Option<SemverRangePrefix>,

    /// Folder where the versioning files are stored.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deferred_version_folder: Option<String>,

    /// Define whether colors are allowed on the standard output.
    ///
    /// The default is to check the terminal capabilities, but you can manually override it to either true or false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_colors: Option<bool>,

    /// Define whether constraints should run on every install.
    ///
    /// If true, Yarn will run your constraints right after finishing its installs. This may help decrease the feedback
    /// loop delay by catching errors long before your CI would even report them.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_constraints_checks: Option<bool>,

    /// Define whether the cache should be shared between all local projects.
    ///
    /// If true (the default), Yarn will store the cache files into a folder located within
    /// [`global_folder`](Self::global_folder) instead of respecting [`cache_folder`](Self::cache_folder).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_global_cache: Option<bool>,

    /// Define whether Yarn should attempt to check for malicious changes.
    ///
    /// If true, Yarn will query the remote registries to validate that the lockfile content matches the remote
    /// information. These checks make installs slower, so you should only run them on branches managed by users
    /// outside your circle of trust.
    ///
    /// Yarn will automatically enable the hardened mode on GitHub pull requests from public repository. Should you
    /// want to disable it, explicitly set it to false in your `.yarnrc.yml` file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_hardened_mode: Option<bool>,

    /// Define whether hyperlinks are allowed on the standard output.
    ///
    /// The default is to check the terminal capabilities, but you can manually override it to either true or false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_hyperlinks: Option<bool>,

    /// Define whether to allow adding/removing files from the cache or not.
    ///
    /// If true, Yarn will refuse to change the cache in any way, whether it would add files or remove them, and will
    /// abort installs instead of letting that happen.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_immutable_cache: Option<bool>,

    /// Define whether to allow adding/removing entries from the lockfile or not.
    ///
    /// If true (the default on CI), Yarn will refuse to change the lockfile in any way, whether it would add new
    /// entries or remove them. Other files can be added to the checklist via the
    /// [`immutable_patterns`](Self::immutable_patterns) setting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_immutable_installs: Option<bool>,

    /// Define whether to print the build output directly within the terminal or not.
    ///
    /// If true (the default on CI environments), Yarn will print the build output directly within the terminal instead
    /// of buffering it in an external log file. Note that by default Yarn will attempt to use collapsible terminal
    /// sequences on supporting CI providers to make the output more legible.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_inline_builds: Option<bool>,

    /// Define whether to print patch hunks directly within the terminal or not.
    ///
    /// If true, Yarn will print any patch sections (hunks) that could not be applied successfully to the terminal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_inline_hunks: Option<bool>,

    /// Define whether to prepend a message name before each printed line or not.
    ///
    /// If true, Yarn will prefix most messages with codes suitable for search engines, with hyperlink support if your
    /// terminal allows it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_message_names: Option<bool>,

    /// Define whether to mirror local cache entries into the global cache or not.
    ///
    /// If true (the default), Yarn will use the global folder as indirection between the network and the actual cache.
    /// This is only useful if [`enable_global_cache`](Self::enable_global_cache) is explicitly set to false, as
    /// otherwise the cache entries are persisted to the global cache no matter what.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_mirror: Option<bool>,

    /// Define whether remote network requests are allowed or not.
    ///
    /// If false, Yarn will never make any request to the network by itself, and will throw an exception rather than
    /// let it happen. It's a very useful setting for CI, which typically want to make sure they aren't loading
    /// their dependencies from the network by mistake.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_network: Option<bool>,

    /// Define whether Yarn should exclusively read package metadata from its cache
    ///
    /// If true, Yarn will replace any network requests by reads from its local caches - even if they contain old
    /// information. This can be useful when performing local work on environments without network access (trains,
    /// planes, ...), as you can at least leverage the packages you installed on the same machine in the past.
    ///
    /// Since this setting will lead to stale data being used, it's recommended to set it for the current session as an
    /// environment variable (by running `export YARN_ENABLE_OFFLINE_MODE=1` in your terminal) rather than by adding it
    /// to your `.yarnrc.yml` file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_offline_mode: Option<bool>,

    /// Define whether animated progress bars should be shown or not.
    ///
    /// If true (the default outside of CI environments), Yarn will show progress bars for long-running events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_progress_bars: Option<bool>,

    /// Define whether to run postinstall scripts or not.
    ///
    /// If false, Yarn will not execute the `postinstall` scripts from third-party packages when installing the project
    /// (workspaces will still see their postinstall scripts evaluated, as they're assumed to be safe if you're running
    /// an install within them).
    ///
    /// Note that you also have the ability to disable scripts on a per-package basis using `dependencies_meta`, or to
    /// re-enable a specific script by combining [`enable_scripts`](Self::enable_scripts) and `dependencies_meta`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_scripts: Option<bool>,

    /// Define whether SSL errors should fail requests or not.
    ///
    /// If false, SSL certificate errors will be ignored
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_strict_ssl: Option<bool>,

    /// Define whether anonymous telemetry data should be sent or not.
    ///
    /// If true (the default outside of CI environments), Yarn will periodically send anonymous data to our servers
    /// tracking some usage information such as the number of dependencies in your project, how many installs you
    /// ran, etc.
    ///
    /// Consult the [Telemetry](https://yarnpkg.com/advanced/telemetry) page for more details about this process.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_telemetry: Option<bool>,

    /// Define whether to print the time spent running each sub-step or not.
    ///
    /// If false, Yarn will not print the time spent running each sub-step when running various commands. This is only
    /// needed for testing purposes, when you want each execution to have exactly the same output as the previous ones.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_timers: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_tips: Option<bool>,

    /// Define whether pure semver ranges should allow workspace resolution or not.
    ///
    /// If false, Yarn won't link workspaces just because their versions happen to match a semver range. Disabling this
    /// setting will require all workspaces to reference one another using the explicit `workspace:` protocol.
    ///
    /// This setting is usually only needed when your project needs to use the published version in order to build the
    /// new one (that's for example what happens with Babel, which depends on the latest stable release to build
    /// the future ones).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_transparent_workspaces: Option<bool>,

    /// Path where all files global to the system will be stored.
    ///
    /// Various files we be stored there: global cache, metadata cache, ...
    #[serde(skip_serializing_if = "Option::is_none")]
    pub global_folder: Option<String>,

    /// Proxy to use when making an HTTP request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_proxy: Option<String>,

    /// Amount of time to wait in seconds before retrying a failed HTTP request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_retry: Option<usize>,

    /// Amount of time to wait before cancelling pending HTTP requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_timeout: Option<String>,

    /// Path to a file containing one or multiple Certificate Authority signing certificates.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub https_ca_file_path: Option<String>,

    /// Path to a file containing a certificate chain in PEM format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub https_cert_file_path: Option<String>,

    /// Path to a file containing a private key in PEM format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub https_key_file_path: Option<String>,

    /// Define a proxy to use when making an HTTPS request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub https_proxy: Option<String>,

    /// Define whether [`yarn_path`](Self::yarn_path) should be respected or not.
    ///
    /// If true, whatever Yarn version is being executed will keep running rather than looking at the value of
    /// [`yarn_path`](Self::yarn_path) to decide.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_path: Option<bool>,

    /// Array of file patterns whose content won't be allowed to change if
    /// [`enable_immutable_installs`](Self::enable_immutable_installs) is set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub immutable_patterns: Option<Vec<String>>,

    /// Scope used when creating packages via the `init` command.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub init_scope: Option<String>,

    /// Additional fields to set when creating packages via the `init` command.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub init_fields: Option<HashMap<String, String>>,

    /// Array of `.env` files which will get injected into any subprocess spawned by Yarn.
    ///
    /// By default Yarn will automatically inject the variables stored in the `.env.yarn` file, but you can use this
    /// setting to change this behavior.
    ///
    /// Note that adding a question mark at the end of the path will silence the error Yarn would throw should the file
    /// be missing, which may come in handy when declaring local configuration files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inject_environment_files: Option<Vec<String>>,

    /// Path where the install state will be persisted.
    ///
    /// The install state file contains a bunch of cached information about your project. It's only used for
    /// optimization purposes, and will be recreated if missing (you don't need to add it to Git).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_state_path: Option<String>,

    /// Alter the log levels for emitted messages.
    ///
    /// This can be used to hide specific messages, or instead make them more prominent. Rules defined there accept
    /// filtering messages by either name or raw content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_filters: Option<Vec<LogFilter>>,

    /// Amount of HTTP requests that are allowed to run at the same time.
    ///
    /// We default to 50 concurrent requests, but it may be required to limit it even more when working behind proxies
    /// that can't handle large amounts of traffic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_concurrency: Option<usize>,

    /// Additional network settings, per hostname.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_settings: Option<HashMap<String, NetworkSetting>>,

    /// Highest point where packages can be hoisted.
    ///
    /// Replacement of the former `nohoist` setting. Possible values are:
    ///
    /// - If [`None`](NmHoistingLimits::None) (the default), packages are hoisted as per the usual rules.
    /// - If [`Workspaces`](NmHoistingLimits::Workspaces), packages won't be hoisted past the workspace that depends on
    ///   them.
    /// - If [`Dependencies`](NmHoistingLimits::Dependencies), transitive dependencies also won't be hoisted past your
    ///   direct dependencies.
    ///
    /// This setting can be overridden on a per-workspace basis using the `installConfig.hoistingLimits` field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nm_hoisting_limits: Option<NmHoistingLimits>,

    /// Define whether workspaces are allowed to require themselves.
    ///
    /// If false, Yarn won't create self-referencing symlinks when using [`NodeLinker::NodeModules`]. This setting can
    /// be overridden on a per-workspace basis using the `installConfig.selfReferences` field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nm_self_references: Option<bool>,

    /// Define how to copy files to their target destination.
    ///
    /// Possible values are:
    ///
    /// - If [`Classic`](NmMode::Classic), regular copy or clone operations are performed.
    /// - If [`HardlinksGlobal`](NmMode::HardlinksGlobal), hardlinks to a global content-addressable store will be
    ///   used.
    /// - If [`HardlinksLocal`](NmMode::HardlinksLocal), hardlinks will only be created between similar packages from
    ///   the same project.
    ///
    /// For compatibility with the ecosystem, the default is [`Classic`](NmMode::Classic).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nm_mode: Option<NmMode>,

    /// Define how Node packages should be installed.
    ///
    /// Yarn supports three ways to install your project's dependencies, based on the
    /// [`node_linker`](Self::node_linker) setting. Possible values are:
    ///
    /// - If [`Pnp`](NodeLinker::Pnp), a single Node.js loader file will be generated.
    /// - If [`Pnpm`](NodeLinker::Pnpm), a `node_modules` will be created using symlinks and hardlinks to a global
    ///   content-addressable store.
    /// - If [`NodeModules`](NodeLinker::NodeModules), a regular `node_modules` folder just like in Yarn Classic or npm
    ///   will be created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_linker: Option<NodeLinker>,

    /// Minimum age of a package version according to the publish date on the npm registry to be considered for
    /// installation.
    ///
    /// If a package version is newer than the minimal age gate, it will not be considered for installation. This can
    /// be used to reduce the likelihood of installing compromised packages, or to avoid relying on packages that
    /// could still be unpublished (e.g. the npm registry has specific rules for packages less than 3 days old).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npm_minimal_age_gate: Option<String>,

    /// Array of package descriptors or package name glob patterns to exclude from all of the package gates.
    ///
    /// If a package descriptor or name matches the specified pattern, it will not be considered when evaluating any of
    /// the package gates.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npm_preapproved_packages: Option<Vec<String>>,

    /// Path where the pnpm store will be stored
    ///
    /// By default, the store is stored in the `node_modules/.store` of the project. Sometimes in CI scenario's it is
    /// convenient to store this in a different location so it can be cached and reused.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pnpm_store_folder: Option<String>,

    /// Define whether to use junctions or symlinks when creating links on Windows.
    ///
    /// Possible values are:
    ///
    /// - If [`Junctions`](WinLinkType::Junctions), Yarn will use Windows junctions when linking workspaces into
    ///   `node_modules` directories, which are always absolute paths.
    /// - If [`Symlinks`](WinLinkType::Symlinks), Yarn will use symlinks, which will use relative paths, and is
    ///   consistent with Yarn's behavior on non-Windows platforms. Symlinks are preferred, but they require the
    ///   Windows user running Yarn to have the create symbolic links privilege. As a result, we default to using
    ///   junctions instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub win_link_type: Option<WinLinkType>,

    /// Define whether to always send authentication credentials when querying the npm registry.
    ///
    /// If true, authentication credentials will always be sent when sending requests to the registries. This shouldn't
    /// be needed unless you configured the registry to reference a private npm mirror.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npm_always_auth: Option<bool>,

    /// Define the registry to use when auditing dependencies.
    ///
    /// If not explicitly set, the value of [`npm_registry_server`](Self::npm_registry_server) will be used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npm_audit_registry: Option<String>,

    /// Define the authentication credentials to use by default when accessing your registries.
    ///
    /// Replacement of the former `_auth` setting. Because it requires storing unencrypted values in your
    /// configuration, [`npm_auth_token`](Self::npm_auth_token) should be preferred when possible.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npm_auth_ident: Option<String>,

    /// Define the authentication token to use by default when accessing your registries.
    ///
    /// Replacement of the former `_authToken` settings. If you're using [`npm_scopes`](Self::npm_scopes) to define
    /// multiple registries, the [`npm_registries`](Self::npm_registries) dictionary allows you to override these
    /// credentials on a per-registry basis.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npm_auth_token: Option<String>,

    /// Define the default access to use when publishing packages to the npm registry.
    ///
    /// Valid values are [`Public`](NpmPublishAccess::Public) and [`Restricted`](NpmPublishAccess::Restricted), but
    /// [`Restricted`](NpmPublishAccess::Restricted) usually requires to register for a paid plan (this is up to the
    /// registry you use). Can be overridden on a per-package basis using the `PublishConfig::access` field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npm_publish_access: Option<NpmPublishAccess>,

    /// Define whether to attach a provenance statement when publishing packages to the npm registry.
    ///
    /// If true, Yarn will generate and publish the provenance information when publishing packages. Can be overridden
    /// on a per-package basis using the `PublishConfig::provenance` field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npm_publish_provenance: Option<bool>,

    /// Array of package name glob patterns to exclude from `yarn npm audit`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npm_audit_exclude_packages: Option<Vec<String>>,

    /// Array of advisory ID glob patterns to ignore from `yarn npm audit` results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npm_audit_ignore_advisories: Option<Vec<String>>,

    /// Define the registry to use when pushing packages.
    ///
    /// If not explicitly set, the value of [`npm_registry_server`](Self::npm_registry_server) will be used. Overridden
    /// by `PublishConfig::registry`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npm_publish_registry: Option<String>,

    /// Per-registry configurations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npm_registries: Option<HashMap<String, NpmRegistry>>,

    /// Define the registry to use when fetching packages.
    ///
    /// Should you want to define different registries for different scopes, see [`npm_scopes`](Self::npm_scopes). To
    /// define the authentication scheme for your servers, see [`npm_auth_token`](Self::npm_auth_token). The url must
    /// use HTTPS by default, but this can be changed by adding it to the
    /// [`unsafe_http_whitelist`](Self::unsafe_http_whitelist).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npm_registry_server: Option<String>,

    /// Per-scope registry configurations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npm_scopes: Option<HashMap<String, NpmScope>>,

    /// Extend the package definitions of your dependencies; useful to fix third-party issues.
    ///
    /// Some packages may have been specified incorrectly with regard to their dependencies - for example with one
    /// dependency being missing, causing Yarn to refuse it the access. The
    /// [`package_extensions`](Self::package_extensions) fields offer a way to extend the existing package
    /// definitions with additional information. If you use it, consider sending a PR upstream and contributing
    /// your extension to the [plugin-compat database](https://github.com/yarnpkg/berry/blob/master/packages/yarnpkg-extensions/sources/index.ts).
    ///
    /// Note: This field is made to add dependencies; if you need to rewrite existing ones, prefer the `resolutions`
    /// field instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_extensions: Option<HashMap<String, PackageExtension>>,

    /// Folder where patch files will be written to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch_folder: Option<String>,

    /// Define whether to generate a Node.js ESM loader or not.
    ///
    /// If true, Yarn will generate an experimental ESM loader (`.pnp.loader.mjs`) on top of the CJS one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pnp_enable_esm_loader: Option<bool>,

    /// Define whether to store the PnP data in the generated file or not.
    ///
    /// If false, Yarn will generate an additional `.pnp.data.json` file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pnp_enable_inlining: Option<bool>,

    /// Define whether to allow packages to rely on the builtin PnP fallback mechanism.
    ///
    /// Possible values are:
    ///
    /// - If [`All`](PnpFallbackMode::All), all packages can access dependencies made available in the fallback.
    /// - If [`DependenciesOnly`](PnpFallbackMode::DependenciesOnly) (the default), dependencies will have access to
    ///   them but not your workspaces.
    /// - If [`None`](PnpFallbackMode::None), no packages will have access to them.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pnp_fallback_mode: Option<PnpFallbackMode>,

    /// Array of file glob patterns that should be forced to use the default CommonJS resolution.
    ///
    /// Files matching those locations will not be covered by PnP and will use the regular Node.js resolution
    /// algorithm. Typically only needed if you have subprojects that aren't yet part of your workspace tree.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pnp_ignore_patterns: Option<Vec<String>>,

    /// Define whether to attempt to simulate traditional `node_modules` hoisting.
    ///
    /// Possible values are:
    ///
    /// - If [`Strict`](PnpMode::Strict) (the default), modules won't be allowed to require packages they don't
    ///   explicitly list in their own dependencies.
    /// - If [`Loose`](PnpMode::Loose), packages will be allowed to access any other package that would have been
    ///   hoisted to the top-level under 1.x installs. Note that, even in loose mode, hoisted require calls are unsafe
    ///   and should be discouraged.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pnp_mode: Option<PnpMode>,

    /// String prepended to the generated PnP loader.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pnp_shebang: Option<String>,

    /// Path where unplugged packages are stored.
    ///
    /// While Yarn attempts to reference and load packages directly from their zip archives, it may not always be
    /// possible. In those cases, Yarn will extract the files to the unplugged folder.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pnp_unplugged_folder: Option<String>,

    /// Define whether to use deferred versioning by default or not.
    ///
    /// If true, deferred versioning by default when running the `yarn version` family of commands.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefer_deferred_versions: Option<bool>,

    /// Define whether to use interactive prompts by default or not.
    ///
    /// If true, Yarn will ask for your guidance when some actions would be improved by being disambiguated. Enabling
    /// this setting also unlocks some features (for example the `yarn add` command will suggest to reuse the same
    /// dependencies as other workspaces if pertinent).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefer_interactive: Option<bool>,

    /// Define whether to reuse most common dependency ranges or not when adding dependencies to a package.
    ///
    /// If true, `yarn add` will attempt to reuse the most common dependency range in other workspaces.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefer_reuse: Option<bool>,

    /// Define whether to truncate lines that would go beyond the size of the terminal or not.
    ///
    /// If true, Yarn will truncate lines that would go beyond the size of the terminal. If progress bars are disabled,
    /// lines will never be truncated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefer_truncated_lines: Option<bool>,

    /// Style of progress bar to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_bar_style: Option<ProgressBarStyle>,

    /// Systems for which Yarn should install packages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_architectures: Option<SupportedArchitectures>,

    /// Maximal amount of concurrent heavy task processing.
    ///
    /// We default to the platform parallelism, but for some CI, `os.cpus` may not report accurate values and may
    /// overwhelm their containers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_pool_concurrency: Option<String>,

    /// Execution strategy for heavy tasks.
    ///
    /// By default will use workers when performing heavy tasks, such as converting tgz files to zip. This setting can
    /// be used to disable workers and use a regular in-thread async processing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_pool_mode: Option<TaskPoolMode>,

    /// Define the minimal amount of time between two telemetry events.
    ///
    /// By default we only send one request per week, making it impossible for us to track your usage with a lower
    /// granularity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub telemetry_interval: Option<String>,

    /// User-defined unique ID to send along with telemetry events.
    ///
    /// The default settings never assign unique IDs to anyone, so we have no way to know which data originates from
    /// which project. This setting can be used to force a user ID to be sent to our telemetry server.
    ///
    /// Frankly, it's only useful in some very specific use cases. For example, we use it on the Yarn repository in
    /// order to exclude our own usage from the public dashboards (since we run Yarn far more often here than
    /// anywhere else, the resulting data would be biased).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub telemetry_user_id: Option<String>,

    /// Define whether to automatically install `@types` dependencies.
    ///
    /// If true, Yarn will automatically add `@types` dependencies when running `yarn add` with packages that don't
    /// provide their own typings (as reported by the Algolia npm database). This behavior is enabled by default if
    /// you have a tsconfig.json file at the root of your project, or in your current workspace.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ts_enable_auto_types: Option<bool>,

    /// Array of hostname glob patterns for which using the HTTP protocol is allowed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unsafe_http_whitelist: Option<Vec<String>>,

    /// Path where virtual packages will be stored.
    ///
    /// Due to a particularity in how Yarn installs packages which list peer dependencies, some packages will be mapped
    /// to multiple virtual directories that don't actually exist on the filesystem. This settings tells Yarn where
    /// to put them. Note that the folder name *must* be `__virtual__`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub virtual_folder: Option<String>,

    /// Path of a Yarn binary to use instead of the global one.
    ///
    /// This binary will be executed instead of any other (including the global one) for any command run within the
    /// directory covered by the rc file. If the file extension ends with `.js` it will be required, and will be
    /// spawned in any other case.
    ///
    /// The [`yarn_path`](Self::yarn_path) setting used to be the preferred way to install Yarn within a project, but
    /// we now recommend to use [Corepack](https://nodejs.org/api/corepack.html) in most cases.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yarn_path: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CacheMigrationMode {
    RequiredOnly,
    MatchSpec,
    Always,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChecksumBehavior {
    Throw,
    Update,
    Ignore,
    Reset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionLevel {
    L0,
    L1,
    L2,
    L3,
    L4,
    L5,
    L6,
    L7,
    L8,
    L9,
    Mixed,
}

impl Serialize for CompressionLevel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::L0 => serializer.serialize_u8(0),
            Self::L1 => serializer.serialize_u8(1),
            Self::L2 => serializer.serialize_u8(2),
            Self::L3 => serializer.serialize_u8(3),
            Self::L4 => serializer.serialize_u8(4),
            Self::L5 => serializer.serialize_u8(5),
            Self::L6 => serializer.serialize_u8(6),
            Self::L7 => serializer.serialize_u8(7),
            Self::L8 => serializer.serialize_u8(8),
            Self::L9 => serializer.serialize_u8(9),
            Self::Mixed => serializer.serialize_str("mixed"),
        }
    }
}

impl<'de> Deserialize<'de> for CompressionLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Either::<u8, String>::deserialize(deserializer)?;
        match value {
            Either::Left(0) => Ok(Self::L0),
            Either::Left(1) => Ok(Self::L1),
            Either::Left(2) => Ok(Self::L2),
            Either::Left(3) => Ok(Self::L3),
            Either::Left(4) => Ok(Self::L4),
            Either::Left(5) => Ok(Self::L5),
            Either::Left(6) => Ok(Self::L6),
            Either::Left(7) => Ok(Self::L7),
            Either::Left(8) => Ok(Self::L8),
            Either::Left(9) => Ok(Self::L9),
            Either::Right(s) if s == "mixed" => Ok(Self::Mixed),
            _ => Err(serde::de::Error::custom("invalid compression level")),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LogFilter {
    Code {
        /// Match all messages with the given code.
        code: String,
        /// New log level to apply to the matching messages. Use [`Discard`](LogLevel::Discard) if you wish to hide
        /// those messages altogether.
        level: LogLevel,
    },
    Text {
        /// Match messages whose content is strictly equal to the given text.
        ///
        /// In case a message matches both `code`-based and `text`-based filters, the `text`-based ones will take
        /// precedence over the `code`-based ones.
        text: String,
        /// New log level to apply to the matching messages. Use [`Discard`](LogLevel::Discard) if you wish to hide
        /// those messages altogether.
        level: LogLevel,
    },
    Pattern {
        /// Match messages whose content match the given glob pattern.
        ///
        /// In case a message matches both `pattern`-based and `code`-based filters, the `pattern`-based ones will take
        /// precedence over the other ones. Patterns can be overridden on a case-by-case basis by using the `text`
        /// filter, which has precedence over `pattern`.
        pattern: String,
        /// New log level to apply to the matching messages. Use [`Discard`](LogLevel::Discard) if you wish to hide
        /// those messages altogether.
        level: LogLevel,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Discard,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkSetting {
    /// See [`enable_network`](YarnRc::enable_network).
    pub enable_network: bool,
    /// See [`http_proxy`](YarnRc::http_proxy).
    pub http_proxy: String,
    /// See [`https_ca_file_path`](YarnRc::https_ca_file_path).
    pub https_ca_file_path: String,
    /// See [`https_cert_file_path`](YarnRc::https_cert_file_path).
    pub https_cert_file_path: String,
    /// See [`https_key_file_path`](YarnRc::https_key_file_path).
    pub https_key_file_path: String,
    /// See [`https_proxy`](YarnRc::https_proxy).
    pub https_proxy: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NmHoistingLimits {
    Workspaces,
    Dependencies,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NmMode {
    Classic,
    HardlinksLocal,
    HardlinksGlobal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NodeLinker {
    NodeModules,
    Pnp,
    Pnpm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NpmPublishAccess {
    Public,
    Restricted,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NpmRegistry {
    /// See [`npm_always_auth`](YarnRc::npm_always_auth).
    pub npm_always_auth: bool,
    /// See [`npm_auth_ident`](YarnRc::npm_auth_ident).
    pub npm_auth_ident: String,
    /// See [`npm_auth_token`](YarnRc::npm_auth_token).
    pub npm_auth_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NpmScope {
    /// See [`npm_publish_registry`](YarnRc::npm_publish_registry).
    pub npm_publish_registry: String,
    /// See [`npm_registry_server`](YarnRc::npm_registry_server).
    pub npm_registry_server: String,
    /// See [`npm_always_auth`](YarnRc::npm_always_auth).
    pub npm_always_auth: bool,
    /// See [`npm_auth_ident`](YarnRc::npm_auth_ident).
    pub npm_auth_ident: String,
    /// See [`npm_auth_token`](YarnRc::npm_auth_token).
    pub npm_auth_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageExtension {
    dependencies: HashMap<String, String>,
    peer_dependencies: HashMap<String, String>,
    peer_dependencies_meta: HashMap<String, PeerDependencyMeta>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeerDependencyMeta {
    optional: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PnpFallbackMode {
    None,
    DependenciesOnly,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PnpMode {
    Strict,
    Loose,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProgressBarStyle {
    Patrick,
    Simba,
    Jack,
    Hogsfather,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemverRangePrefix {
    Caret,
    Tilde,
    None,
}

impl Serialize for SemverRangePrefix {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Caret => serializer.serialize_str("^"),
            Self::Tilde => serializer.serialize_str("~"),
            Self::None => serializer.serialize_str(""),
        }
    }
}

impl<'de> Deserialize<'de> for SemverRangePrefix {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "^" => Ok(Self::Caret),
            "~" => Ok(Self::Tilde),
            "" => Ok(Self::None),
            _ => Err(serde::de::Error::custom("invalid semver range prefix")),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportedArchitectures {
    /// List of operating systems to cover.
    pub os: Vec<String>,
    /// List of CPU architectures to cover.
    ///
    /// See <https://nodejs.org/docs/latest/api/process.html#processarch> for the architectures supported by Node.js.
    pub cpu: Vec<String>,
    /// The list of standard C libraries to cover.
    pub libc: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskPoolMode {
    Async,
    Workers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WinLinkType {
    Junctions,
    Symlinks,
}
