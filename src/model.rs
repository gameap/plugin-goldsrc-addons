//! JSON request/response DTOs of the plugin API.

use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

#[derive(Deserialize, Debug)]
pub struct ToggleRequest {
    pub file: String,
    pub enabled: bool,
}

#[derive(Deserialize, Debug)]
pub struct AddPluginRequest {
    pub file: String,
    #[serde(default = "default_true")]
    pub enable: bool,
    /// Metamod only: mod-dir-relative location of the uploaded file.
    /// Defaults to `addons/<stem>/<file>`.
    #[serde(default)]
    pub path: Option<String>,
    /// Overwrite an already registered entry instead of failing with 409.
    #[serde(default)]
    pub force: bool,
}

#[derive(Deserialize, Debug)]
pub struct RemovePluginRequest {
    pub file: String,
}

#[derive(Serialize, Debug)]
pub struct StateResponse {
    pub server_id: u64,
    pub game_code: String,
    pub engine: String,
    pub mod_dir: String,
    pub paths: StatePaths,
    pub metamod: MetamodState,
    pub amxx: AmxxState,
}

/// All paths are relative to the server directory, ready to be passed to the
/// panel file-manager API as-is.
#[derive(Serialize, Debug)]
pub struct StatePaths {
    pub liblist: String,
    pub metamod_dir: String,
    pub metamod_plugins_ini: String,
    pub amxx_dir: String,
    pub amxx_plugins_ini: String,
    pub amxx_plugins_dir: String,
    pub amxx_configs_dir: String,
}

#[derive(Serialize, Debug)]
pub struct MetamodState {
    /// liblist.gam points into addons/<metamod dir>.
    pub installed: bool,
    /// The addons directory exists even though liblist does not point at it.
    pub dir_present: bool,
    pub plugins_ini_exists: bool,
    pub plugins: Vec<MetamodPluginEntry>,
}

#[derive(Serialize, Debug)]
pub struct MetamodPluginEntry {
    pub platform: String,
    /// Path exactly as written in plugins.ini.
    pub path: String,
    pub file: String,
    /// Free-text trailing description (the metamod analogue of a comment).
    pub description: Option<String>,
    pub enabled: bool,
    pub missing: bool,
    /// The AMXX loader entry — managed by the platform, locked in the UI.
    pub system: bool,
    /// Index of the display group; unnamed entries share one trailing "Other".
    pub group_index: u32,
    /// Header of the display group, `None` for the common "Other" group.
    pub group_title: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct AmxxState {
    pub installed: bool,
    pub registered_in_metamod: bool,
    pub plugins_ini_exists: bool,
    pub plugins: Vec<AmxxPluginEntry>,
}

#[derive(Serialize, Debug)]
pub struct AmxxPluginEntry {
    pub file: String,
    /// The AMX Mod X `debug` load flag.
    pub debug: bool,
    /// Inline `; comment` after the entry.
    pub comment: Option<String>,
    pub enabled: bool,
    pub missing: bool,
    pub has_config: bool,
    pub config_path: Option<String>,
    /// Index of the display group; unnamed entries share one trailing "Other".
    pub group_index: u32,
    /// Header of the display group, `None` for the common "Other" group.
    pub group_title: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct ToggleResponse {
    pub file: String,
    pub enabled: bool,
    pub changed: bool,
}

#[derive(Serialize, Debug)]
pub struct AddPluginResponse {
    pub file: String,
    pub enabled: bool,
    pub line: String,
    /// True when an existing entry was overwritten (force install).
    pub replaced: bool,
}

#[derive(Serialize, Debug)]
pub struct RemovePluginResponse {
    pub file: String,
    pub entry_removed: bool,
    pub file_deleted: bool,
}

#[derive(Deserialize, Debug)]
pub struct SetAttributesRequest {
    pub file: String,
    /// AMX Mod X `debug` load flag (ignored for Metamod entries).
    #[serde(default)]
    pub debug: bool,
    /// Full desired inline comment; `null`/absent clears it.
    #[serde(default)]
    pub comment: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct SetAttributesResponse {
    pub file: String,
    pub debug: bool,
    pub comment: Option<String>,
    pub changed: bool,
}
