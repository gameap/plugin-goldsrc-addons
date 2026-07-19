pub mod ini;
pub mod liblist;
pub mod moddir;
pub mod paths;

/// Managed plugin platform.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Platform {
    Metamod,
    Amxx,
}

impl Platform {
    pub fn from_route_param(value: &str) -> Option<Platform> {
        match value {
            "metamod" => Some(Platform::Metamod),
            "amxx" => Some(Platform::Amxx),
            _ => None,
        }
    }

    pub fn dialect(self) -> ini::Dialect {
        match self {
            Platform::Metamod => ini::Dialect::Metamod,
            Platform::Amxx => ini::Dialect::Amxx,
        }
    }
}

/// AMXX layout relative to the mod dir.
pub const AMXX_DIR: &str = "addons/amxmodx";
pub const AMXX_PLUGINS_DIR: &str = "addons/amxmodx/plugins";
pub const AMXX_CONFIGS_DIR: &str = "addons/amxmodx/configs";
pub const AMXX_PLUGINS_INI: &str = "addons/amxmodx/configs/plugins.ini";
/// Default metamod dir when liblist.gam does not point at one.
pub const METAMOD_DIR_DEFAULT: &str = "addons/metamod";
pub const LIBLIST_FILE: &str = "liblist.gam";

/// The AMXX loader entry in metamod's plugins.ini is managed by the platform,
/// not by this plugin's list operations.
pub fn is_amxx_loader_entry(path_or_file: &str) -> bool {
    paths::file_name(&paths::normalize_slashes(path_or_file))
        .to_ascii_lowercase()
        .starts_with("amxmodx_mm")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amxx_loader_entry_detection() {
        assert!(is_amxx_loader_entry("addons/amxmodx/dlls/amxmodx_mm_i386.so"));
        assert!(is_amxx_loader_entry("amxmodx_mm.dll"));
        assert!(!is_amxx_loader_entry("addons/reunion/reunion_mm_i386.so"));
    }
}
