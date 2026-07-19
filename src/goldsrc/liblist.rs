//! Minimal `liblist.gam` parser: `key value` / `key "value"` lines,
//! `//` comments. Only the gamedll keys are of interest — they tell whether
//! Metamod is wired into the mod.

use crate::goldsrc::paths;

#[derive(Clone, Debug, Default)]
pub struct Liblist {
    pub gamedll: Option<String>,
    pub gamedll_linux: Option<String>,
    pub gamedll_osx: Option<String>,
}

pub fn parse(bytes: &[u8]) -> Liblist {
    let mut out = Liblist::default();
    for raw_line in bytes.split(|b| *b == b'\n') {
        let line = String::from_utf8_lossy(raw_line);
        let line = line.trim().trim_end_matches('\r').trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        let Some((key, value)) = split_key_value(line) else {
            continue;
        };
        let value = value.trim().trim_matches('"').to_string();
        match key.to_ascii_lowercase().as_str() {
            "gamedll" => out.gamedll = Some(value),
            "gamedll_linux" => out.gamedll_linux = Some(value),
            "gamedll_osx" => out.gamedll_osx = Some(value),
            _ => {}
        }
    }
    out
}

fn split_key_value(line: &str) -> Option<(&str, &str)> {
    let idx = line.find([' ', '\t'])?;
    Some((&line[..idx], &line[idx + 1..]))
}

/// Returns the metamod addons directory ("addons/<dir>") when any gamedll
/// entry points into it. Catches metamod, metamod-p and metamod-r layouts.
pub fn metamod_dir_from(liblist: &Liblist) -> Option<String> {
    [
        liblist.gamedll_linux.as_deref(),
        liblist.gamedll.as_deref(),
        liblist.gamedll_osx.as_deref(),
    ]
    .into_iter()
    .flatten()
    .find_map(metamod_dir_from_path)
}

fn metamod_dir_from_path(path: &str) -> Option<String> {
    let normalized = paths::normalize_slashes(path);
    let mut segments = normalized.split('/');
    let first = segments.next()?;
    let second = segments.next()?;
    if first.eq_ignore_ascii_case("addons") && second.to_ascii_lowercase().contains("metamod") {
        Some(format!("{first}/{second}"))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIBLIST: &[u8] = br#"// Half-Life game dll description
game "Counter-Strike"
url_info "www.counter-strike.net"
type "multiplayer_only"
version "1.6"
mpentity "info_player_start"
gamedll "dlls\mp.dll"
gamedll_linux "dlls/cs.so"
gamedll_osx "dlls/cs.dylib"
"#;

    #[test]
    fn parses_vanilla_liblist() {
        let liblist = parse(LIBLIST);
        assert_eq!(liblist.gamedll.as_deref(), Some("dlls\\mp.dll"));
        assert_eq!(liblist.gamedll_linux.as_deref(), Some("dlls/cs.so"));
        assert_eq!(metamod_dir_from(&liblist), None);
    }

    #[test]
    fn detects_metamod_variants() {
        let liblist = parse(b"gamedll_linux \"addons/metamod/dlls/metamod_i386.so\"\n");
        assert_eq!(metamod_dir_from(&liblist).as_deref(), Some("addons/metamod"));

        let liblist = parse(b"gamedll \"addons\\metamod-p\\dlls\\metamod.dll\"\n");
        assert_eq!(
            metamod_dir_from(&liblist).as_deref(),
            Some("addons/metamod-p")
        );

        let liblist = parse(b"gamedll_linux addons/Metamod-r/dlls/metamod_i386.so\n");
        assert_eq!(
            metamod_dir_from(&liblist).as_deref(),
            Some("addons/Metamod-r")
        );
    }

    #[test]
    fn cp1251_and_crlf_tolerated() {
        let mut input = Vec::new();
        input.extend_from_slice(b"// \xEA\xEE\xEC\xEC\xE5\xED\xF2\r\n");
        input.extend_from_slice(b"gamedll_linux \"addons/metamod/dlls/metamod_i386.so\"\r\n");
        let liblist = parse(&input);
        assert_eq!(metamod_dir_from(&liblist).as_deref(), Some("addons/metamod"));
    }
}
