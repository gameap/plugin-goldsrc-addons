//! GoldSource mod directory resolution: game code → mod folder inside the
//! server root (the folder that holds `liblist.gam`).

/// Known GameAP game codes for GoldSource games.
pub fn known_mod_dir(game_code: &str) -> Option<&'static str> {
    match game_code.to_ascii_lowercase().as_str() {
        "cstrike" | "cs" | "cs16" => Some("cstrike"),
        "valve" | "hl" | "hldm" => Some("valve"),
        "czero" | "cscz" => Some("czero"),
        "dod" => Some("dod"),
        "tfc" => Some("tfc"),
        "gearbox" | "op4" | "opfor" => Some("gearbox"),
        "dmc" => Some("dmc"),
        "ricochet" => Some("ricochet"),
        "ag" => Some("ag"),
        "svencoop" => Some("svencoop"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_known_codes() {
        assert_eq!(known_mod_dir("cstrike"), Some("cstrike"));
        assert_eq!(known_mod_dir("CS"), Some("cstrike"));
        assert_eq!(known_mod_dir("valve"), Some("valve"));
        assert_eq!(known_mod_dir("hl"), Some("valve"));
        assert_eq!(known_mod_dir("op4"), Some("gearbox"));
        assert_eq!(known_mod_dir("rust"), None);
    }
}
