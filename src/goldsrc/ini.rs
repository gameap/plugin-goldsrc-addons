//! Byte-preserving round-trip document for GoldSource `plugins.ini` files.
//!
//! Real-world files are frequently CRLF, may start with a UTF-8 BOM and often
//! contain CP1251 comments, so the document never converts whole files to
//! UTF-8. Lines are kept as raw bytes (terminator included); serialization is
//! plain concatenation, and edits touch only the `;` marker or the edited line.

use crate::goldsrc::paths;

const BOM: &[u8] = b"\xEF\xBB\xBF";

const METAMOD_PLATFORMS: &[&str] = &["linux", "win32", "lin32", "lin64", "win64", "osx", "mac"];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dialect {
    Metamod,
    Amxx,
}

#[derive(Clone, Debug)]
pub struct Document {
    dialect: Dialect,
    lines: Vec<Line>,
}

#[derive(Clone, Debug)]
struct Line {
    /// Full line bytes including its terminator ("\n", "\r\n" or none for the last line).
    raw: Vec<u8>,
    kind: LineKind,
}

#[derive(Clone, Debug)]
enum LineKind {
    Entry(Entry),
    Other,
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub enabled: bool,
    /// Where the content begins in `raw` (after indent, BOM and, when
    /// disabled, the comment-marker run).
    body_start: usize,
    /// Start of the marker run (equals `body_start` when enabled).
    marker_start: usize,
    pub fields: EntryFields,
}

#[derive(Clone, Debug)]
pub enum EntryFields {
    Metamod {
        platform: String,
        path: String,
        rest: String,
    },
    Amxx {
        file: String,
        rest: String,
    },
}

impl EntryFields {
    /// The key the entry is addressed by: file name for AMXX,
    /// path basename for Metamod.
    pub fn key(&self) -> &str {
        match self {
            EntryFields::Amxx { file, .. } => file,
            EntryFields::Metamod { path, .. } => paths::file_name(path),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct EntryNotFound;

/// A run of entries grouped for display by their `;`-header comment. `title` is
/// `None` for entries not under any header (rendered as a common "Other" group).
pub struct EntryGroup<'a> {
    pub title: Option<String>,
    pub entries: Vec<&'a Entry>,
}

impl Document {
    /// Never fails: lines that do not look like plugin entries round-trip
    /// verbatim as `Other`.
    pub fn parse(dialect: Dialect, bytes: &[u8]) -> Document {
        let mut lines = Vec::new();
        let mut rest = bytes;
        while !rest.is_empty() {
            let end = match rest.iter().position(|b| *b == b'\n') {
                Some(idx) => idx + 1,
                None => rest.len(),
            };
            let raw = rest[..end].to_vec();
            rest = &rest[end..];
            let kind = classify(dialect, &raw);
            lines.push(Line { raw, kind });
        }
        Document { dialect, lines }
    }

    /// Byte-identical for untouched documents.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.lines.iter().map(|l| l.raw.len()).sum());
        for line in &self.lines {
            out.extend_from_slice(&line.raw);
        }
        out
    }

    pub fn entries(&self) -> impl Iterator<Item = &Entry> {
        self.lines.iter().filter_map(|line| match &line.kind {
            LineKind::Entry(entry) => Some(entry),
            LineKind::Other => None,
        })
    }

    /// Partitions the entries into display groups (file order, only groups with
    /// at least one entry). Blocks are separated by blank lines and named by
    /// their leading `;`-header. Two refinements over plain blocks make the
    /// grouping robust to real files: a comment appearing *after* entries starts
    /// a new group, and a header with no entries of its own carries forward to
    /// the next header-less block.
    pub fn groups(&self) -> Vec<EntryGroup<'_>> {
        let mut out: Vec<EntryGroup<'_>> = Vec::new();
        let mut title: Option<String> = None;
        let mut entries: Vec<&Entry> = Vec::new();
        let mut pending: Option<String> = None;

        for line in &self.lines {
            match &line.kind {
                LineKind::Entry(entry) => {
                    if entries.is_empty() && title.is_none() {
                        title = pending.take();
                    }
                    entries.push(entry);
                }
                LineKind::Other if is_blank_line(&line.raw) => {
                    if !entries.is_empty() {
                        out.push(EntryGroup {
                            title: title.take(),
                            entries: std::mem::take(&mut entries),
                        });
                        pending = None;
                    } else if title.is_some() {
                        // A header with no entries of its own — remember it for
                        // the next header-less block.
                        pending = title.take();
                    }
                }
                LineKind::Other => {
                    let Some(text) = comment_text(&line.raw) else {
                        continue; // blankless divider like `;` or `;;;;`
                    };
                    if !entries.is_empty() {
                        // A comment after entries opens a new section.
                        out.push(EntryGroup {
                            title: title.take(),
                            entries: std::mem::take(&mut entries),
                        });
                        title = Some(text);
                        pending = None;
                    } else if title.is_none() {
                        title = Some(text);
                        pending = None;
                    }
                }
            }
        }
        if !entries.is_empty() {
            out.push(EntryGroup {
                title: title.take(),
                entries,
            });
        }
        out
    }

    /// AMXX: exact file-name match. Metamod: basename or full ini path match.
    pub fn find_entry(&self, key: &str) -> Option<usize> {
        self.lines.iter().position(|line| match &line.kind {
            LineKind::Entry(entry) => entry_matches(entry, key),
            LineKind::Other => false,
        })
    }

    pub fn entry_at(&self, index: usize) -> Option<&Entry> {
        match self.lines.get(index).map(|l| &l.kind) {
            Some(LineKind::Entry(entry)) => Some(entry),
            _ => None,
        }
    }

    /// Ok(true) — modified, Ok(false) — already in the requested state.
    pub fn set_enabled(&mut self, key: &str, enabled: bool) -> Result<bool, EntryNotFound> {
        let index = self.find_entry(key).ok_or(EntryNotFound)?;
        let line = &mut self.lines[index];
        let LineKind::Entry(entry) = &line.kind else {
            return Err(EntryNotFound);
        };
        if entry.enabled == enabled {
            return Ok(false);
        }
        if enabled {
            line.raw.drain(entry.marker_start..entry.body_start);
        } else {
            line.raw.insert(entry.marker_start, b';');
        }
        line.kind = classify(self.dialect, &line.raw);
        debug_assert!(matches!(line.kind, LineKind::Entry(_)));
        Ok(true)
    }

    /// Appends an entry line using the document's dominant line terminator.
    pub fn append_entry(&mut self, body: &str, enabled: bool) {
        let terminator = self.dominant_terminator();
        if let Some(last) = self.lines.last_mut()
            && !last.raw.ends_with(b"\n")
        {
            last.raw.extend_from_slice(terminator);
        }
        let mut raw = Vec::with_capacity(body.len() + 3);
        if !enabled {
            raw.push(b';');
        }
        raw.extend_from_slice(body.as_bytes());
        raw.extend_from_slice(terminator);
        let kind = classify(self.dialect, &raw);
        self.lines.push(Line { raw, kind });
    }

    /// Rewrites the entry line in place (same position, same terminator) and
    /// returns the previous entry.
    pub fn replace_entry(
        &mut self,
        key: &str,
        body: &str,
        enabled: bool,
    ) -> Result<Entry, EntryNotFound> {
        let index = self.find_entry(key).ok_or(EntryNotFound)?;
        let line = &mut self.lines[index];
        let LineKind::Entry(old) = line.kind.clone() else {
            return Err(EntryNotFound);
        };
        let terminator_start = line.raw.len() - terminator_len(&line.raw);
        let terminator = line.raw[terminator_start..].to_vec();
        let mut raw = Vec::with_capacity(body.len() + terminator.len() + 1);
        if !enabled {
            raw.push(b';');
        }
        raw.extend_from_slice(body.as_bytes());
        raw.extend_from_slice(&terminator);
        line.raw = raw;
        line.kind = classify(self.dialect, &line.raw);
        debug_assert!(matches!(line.kind, LineKind::Entry(_)));
        Ok(old)
    }

    /// Removes the whole entry line and returns the parsed entry.
    pub fn remove_entry(&mut self, key: &str) -> Result<Entry, EntryNotFound> {
        let index = self.find_entry(key).ok_or(EntryNotFound)?;
        let line = self.lines.remove(index);
        match line.kind {
            LineKind::Entry(entry) => Ok(entry),
            LineKind::Other => Err(EntryNotFound),
        }
    }

    fn dominant_terminator(&self) -> &'static [u8] {
        let crlf = self
            .lines
            .iter()
            .filter(|l| l.raw.ends_with(b"\r\n"))
            .count();
        let lf = self
            .lines
            .iter()
            .filter(|l| l.raw.ends_with(b"\n"))
            .count()
            - crlf;
        if crlf > lf { b"\r\n" } else { b"\n" }
    }
}

fn entry_matches(entry: &Entry, key: &str) -> bool {
    match &entry.fields {
        EntryFields::Amxx { file, .. } => file.eq_ignore_ascii_case(key),
        EntryFields::Metamod { path, .. } => {
            let normalized = paths::normalize_slashes(path);
            paths::file_name(&normalized).eq_ignore_ascii_case(key)
                || normalized.eq_ignore_ascii_case(&paths::normalize_slashes(key))
        }
    }
}

fn classify(dialect: Dialect, raw: &[u8]) -> LineKind {
    let content = &raw[..raw.len() - terminator_len(raw)];
    let (marker_start, body_start) = marker_body_offsets(content);
    match parse_fields(dialect, &content[body_start..]) {
        Some(fields) => LineKind::Entry(Entry {
            enabled: marker_start == body_start,
            body_start,
            marker_start,
            fields,
        }),
        None => LineKind::Other,
    }
}

/// Offsets (into `content`, i.e. `raw` minus terminator) of the comment-marker
/// run and of the body that follows it, skipping a leading BOM and indentation.
/// When the line has no `;`/`#`/`//` marker the two offsets are equal (the first
/// non-indent byte), which is what makes an entry "enabled".
fn marker_body_offsets(content: &[u8]) -> (usize, usize) {
    let mut idx = 0;
    if content.starts_with(BOM) {
        idx = BOM.len();
    }
    while idx < content.len() && (content[idx] == b' ' || content[idx] == b'\t') {
        idx += 1;
    }
    let marker_start = idx;

    let mut body_start = idx;
    if idx < content.len()
        && (content[idx] == b';' || content[idx] == b'#' || content[idx..].starts_with(b"//"))
    {
        while body_start < content.len()
            && (content[body_start] == b';'
                || content[body_start] == b'#'
                || content[body_start] == b'/')
        {
            body_start += 1;
        }
        while body_start < content.len()
            && (content[body_start] == b' ' || content[body_start] == b'\t')
        {
            body_start += 1;
        }
    }
    (marker_start, body_start)
}

/// The line's content without its terminator or a leading BOM.
fn content_span(raw: &[u8]) -> &[u8] {
    let content = &raw[..raw.len() - terminator_len(raw)];
    content.strip_prefix(BOM).unwrap_or(content)
}

/// A line that is empty or whitespace-only — a blank-line group separator.
fn is_blank_line(raw: &[u8]) -> bool {
    content_span(raw).iter().all(|b| *b == b' ' || *b == b'\t')
}

/// Header text of a comment-only line — the text after the marker run, trimmed.
/// `None` when the line has no comment marker or is empty after it (`;`, `;;;;`).
fn comment_text(raw: &[u8]) -> Option<String> {
    let content = &raw[..raw.len() - terminator_len(raw)];
    let (marker_start, body_start) = marker_body_offsets(content);
    if marker_start == body_start {
        return None;
    }
    let text = String::from_utf8_lossy(&content[body_start..]);
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_string())
}

fn terminator_len(raw: &[u8]) -> usize {
    if raw.ends_with(b"\r\n") {
        2
    } else if raw.ends_with(b"\n") {
        1
    } else {
        0
    }
}

fn parse_fields(dialect: Dialect, body: &[u8]) -> Option<EntryFields> {
    let tokens = tokenize(body);
    let first = tokens.first()?;
    match dialect {
        Dialect::Amxx => {
            if !first.to_ascii_lowercase().ends_with(".amxx") {
                return None;
            }
            Some(EntryFields::Amxx {
                file: first.clone(),
                rest: tokens[1..].join(" "),
            })
        }
        Dialect::Metamod => {
            let platform = first.to_ascii_lowercase();
            if !METAMOD_PLATFORMS.contains(&platform.as_str()) {
                return None;
            }
            let path = tokens.get(1)?.clone();
            Some(EntryFields::Metamod {
                platform,
                path,
                rest: tokens[2..].join(" "),
            })
        }
    }
}

fn tokenize(body: &[u8]) -> Vec<String> {
    body.split(|b| *b == b' ' || *b == b'\t')
        .filter(|t| !t.is_empty())
        .map(|t| String::from_utf8_lossy(t).into_owned())
        .collect()
}

/// Splits an AMXX entry's trailing `rest` (everything after the file name) into
/// the `debug` flag and its inline comment. `debug` is true only when the first
/// token is exactly `debug`, matching the AMX Mod X loader (`strcmp`).
pub fn amxx_debug_comment(rest: &str) -> (bool, Option<String>) {
    let rest = rest.trim_start();
    let first_end = rest.find([' ', '\t']).unwrap_or(rest.len());
    let (debug, after) = if &rest[..first_end] == "debug" {
        (true, &rest[first_end..])
    } else {
        (false, rest)
    };
    (debug, extract_inline_comment(after))
}

/// Text of an inline comment inside `s` — everything after the first `;`, `#` or
/// `//` marker run, trimmed. `None` when there is no marker or it is empty.
fn extract_inline_comment(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let start = (0..bytes.len())
        .find(|&i| bytes[i] == b';' || bytes[i] == b'#' || s[i..].starts_with("//"))?;
    let mut body = start;
    while body < bytes.len()
        && (bytes[body] == b';' || bytes[body] == b'#' || bytes[body] == b'/')
    {
        body += 1;
    }
    let text = s[body..].trim();
    (!text.is_empty()).then(|| text.to_string())
}

/// Canonical AMXX entry body: `file`, then ` debug` when set, then ` ; comment`
/// when non-empty. Used to rewrite a line when its debug flag or comment change.
pub fn amxx_meta_body(file: &str, debug: bool, comment: Option<&str>) -> String {
    let mut body = String::from(file);
    if debug {
        body.push_str(" debug");
    }
    if let Some(comment) = comment.map(str::trim).filter(|c| !c.is_empty()) {
        body.push_str(" ; ");
        body.push_str(comment);
    }
    body
}

#[cfg(test)]
mod tests {
    use super::*;

    const AMXX_INI: &[u8] = b"; AMX Mod X plugins\n\nadmin.amxx\t\t; admin base\nadmincmd.amxx\n;statsx.amxx\ncsdm_main.amxx debug\n";

    #[test]
    fn round_trip_is_byte_identical() {
        let doc = Document::parse(Dialect::Amxx, AMXX_INI);
        assert_eq!(doc.to_bytes(), AMXX_INI);
    }

    #[test]
    fn round_trip_crlf_bom_cp1251() {
        // "; комментарий" in CP1251 bytes + BOM + CRLF + no trailing newline.
        let mut input = Vec::new();
        input.extend_from_slice(b"\xEF\xBB\xBF; \xEA\xEE\xEC\xEC\xE5\xED\xF2\xE0\xF0\xE8\xE9\r\n");
        input.extend_from_slice(b"admin.amxx\r\n");
        input.extend_from_slice(b";statsx.amxx");
        let doc = Document::parse(Dialect::Amxx, &input);
        assert_eq!(doc.to_bytes(), input);
        let entries: Vec<_> = doc.entries().collect();
        assert_eq!(entries.len(), 2);
        assert!(entries[0].enabled);
        assert!(!entries[1].enabled);
    }

    #[test]
    fn bom_line_entry_is_recognized() {
        let mut input = Vec::new();
        input.extend_from_slice(b"\xEF\xBB\xBFadmin.amxx\n");
        let doc = Document::parse(Dialect::Amxx, &input);
        assert_eq!(doc.entries().count(), 1);
        assert_eq!(doc.to_bytes(), input);
    }

    #[test]
    fn classifies_comments_vs_disabled_entries() {
        let doc = Document::parse(Dialect::Amxx, AMXX_INI);
        let entries: Vec<_> = doc.entries().collect();
        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0].fields.key(), "admin.amxx");
        assert!(entries[0].enabled);
        assert_eq!(entries[2].fields.key(), "statsx.amxx");
        assert!(!entries[2].enabled);
        match &entries[3].fields {
            EntryFields::Amxx { file, rest } => {
                assert_eq!(file, "csdm_main.amxx");
                assert_eq!(rest, "debug");
            }
            _ => panic!("expected amxx fields"),
        }
    }

    #[test]
    fn double_marker_entry_is_disabled_entry() {
        let doc = Document::parse(Dialect::Amxx, b";; statsx.amxx\n;\tparachute.amxx\n");
        let entries: Vec<_> = doc.entries().collect();
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|e| !e.enabled));
    }

    #[test]
    fn toggle_disable_and_enable_round_trip() {
        let mut doc = Document::parse(Dialect::Amxx, AMXX_INI);
        assert_eq!(doc.set_enabled("admin.amxx", false), Ok(true));
        let bytes = doc.to_bytes();
        assert!(bytes.windows(11).any(|w| w == b";admin.amxx"));

        // Idempotent.
        assert_eq!(doc.set_enabled("admin.amxx", false), Ok(false));

        assert_eq!(doc.set_enabled("admin.amxx", true), Ok(true));
        // The tab and trailing comment on the admin.amxx line must be intact.
        assert_eq!(doc.to_bytes(), AMXX_INI);
    }

    #[test]
    fn enable_strips_marker_run() {
        let mut doc = Document::parse(Dialect::Amxx, b";; \tstatsx.amxx\n");
        assert_eq!(doc.set_enabled("statsx.amxx", true), Ok(true));
        assert_eq!(doc.to_bytes(), b"statsx.amxx\n");
    }

    #[test]
    fn toggle_missing_entry() {
        let mut doc = Document::parse(Dialect::Amxx, AMXX_INI);
        assert_eq!(doc.set_enabled("nope.amxx", true), Err(EntryNotFound));
    }

    #[test]
    fn metamod_dialect_parsing() {
        let ini = b"; Metamod plugins\nlinux addons/amxmodx/dlls/amxmodx_mm_i386.so\nwin32 addons\\reunion\\reunion_mm.dll Reunion\n;linux addons/whblocker/whblocker_mm_i386.so\nrandom garbage line\n";
        let doc = Document::parse(Dialect::Metamod, ini);
        assert_eq!(doc.to_bytes(), ini);
        let entries: Vec<_> = doc.entries().collect();
        assert_eq!(entries.len(), 3);
        match &entries[0].fields {
            EntryFields::Metamod { platform, path, .. } => {
                assert_eq!(platform, "linux");
                assert_eq!(path, "addons/amxmodx/dlls/amxmodx_mm_i386.so");
            }
            _ => panic!("expected metamod fields"),
        }
        match &entries[1].fields {
            EntryFields::Metamod { platform, path, rest } => {
                assert_eq!(platform, "win32");
                assert_eq!(path, "addons\\reunion\\reunion_mm.dll");
                assert_eq!(rest, "Reunion");
            }
            _ => panic!("expected metamod fields"),
        }
        assert!(!entries[2].enabled);
    }

    #[test]
    fn metamod_find_by_basename_and_path() {
        let doc = Document::parse(
            Dialect::Metamod,
            b"linux addons/reunion/reunion_mm_i386.so\n",
        );
        assert!(doc.find_entry("reunion_mm_i386.so").is_some());
        assert!(doc.find_entry("addons/reunion/reunion_mm_i386.so").is_some());
        assert!(doc.find_entry("other.so").is_none());
    }

    #[test]
    fn append_entry_matches_terminator_and_fixes_last_line() {
        let mut doc = Document::parse(Dialect::Amxx, b"admin.amxx\r\n;statsx.amxx");
        doc.append_entry("parachute.amxx", true);
        assert_eq!(
            doc.to_bytes(),
            b"admin.amxx\r\n;statsx.amxx\r\nparachute.amxx\r\n"
        );

        let mut doc = Document::parse(Dialect::Amxx, b"");
        doc.append_entry("admin.amxx", false);
        assert_eq!(doc.to_bytes(), b";admin.amxx\n");
        assert!(!doc.entries().next().map(|e| e.enabled).unwrap_or(true));
    }

    #[test]
    fn replace_entry_keeps_position_and_terminator() {
        let ini = b"linux addons/amxmodx/dlls/amxmodx_mm_i386.so\r\nlinux addons/reunion/old_reunion.so\r\nlinux addons/whb/whb.so\r\n";
        let mut doc = Document::parse(Dialect::Metamod, ini);
        let old = doc
            .replace_entry("old_reunion.so", "linux addons/reunion/reunion_mm_i386.so", true)
            .expect("entry exists");
        assert_eq!(old.fields.key(), "old_reunion.so");
        assert_eq!(
            doc.to_bytes(),
            b"linux addons/amxmodx/dlls/amxmodx_mm_i386.so\r\nlinux addons/reunion/reunion_mm_i386.so\r\nlinux addons/whb/whb.so\r\n"
        );

        doc.replace_entry("whb.so", "linux addons/whb/whb.so", false)
            .expect("entry exists");
        assert!(doc.to_bytes().ends_with(b";linux addons/whb/whb.so\r\n"));
    }

    #[test]
    fn replace_missing_entry() {
        let mut doc = Document::parse(Dialect::Amxx, AMXX_INI);
        assert!(doc.replace_entry("nope.amxx", "nope.amxx", true).is_err());
    }

    #[test]
    fn remove_entry_deletes_line() {
        let mut doc = Document::parse(Dialect::Amxx, AMXX_INI);
        let entry = doc.remove_entry("admincmd.amxx").expect("entry exists");
        assert_eq!(entry.fields.key(), "admincmd.amxx");
        let bytes = doc.to_bytes();
        assert!(!bytes.windows(13).any(|w| w == b"admincmd.amxx"));
        assert!(bytes.windows(10).any(|w| w == b"admin.amxx"));
    }

    #[test]
    fn amxx_case_insensitive_extension() {
        let doc = Document::parse(Dialect::Amxx, b"Admin.AMXX\n");
        assert_eq!(doc.entries().count(), 1);
        assert!(doc.find_entry("admin.amxx").is_some());
    }

    // Condensed version of a real plugins.ini exercising every grouping case:
    // a file banner, named sections, a disabled entry that *looks* like a header
    // (`; restrictnames.amxx`), a mid-block header split, a header orphaned by a
    // blank line (`; Configuration`), and header-less/lone plugins.
    const GROUPING_INI: &[u8] = b"; AMX Mod X plugins\n\
\n\
; AMXBans 1.6 GM\n\
amxbans_core.amxx\n\
amxbans_main.amxx\n\
\n\
; restrictnames.amxx\n\
; Admin Base\n\
;admin.amxx\t\t; admin base\n\
;admin_sql.amxx\n\
\n\
; Basic\n\
admincmd.amxx\t\t; basic admin console commands\n\
\n\
; Half-Life GunGame\n\
gungame.amxx\t\t\tdebug\n\
gg_respawnItems.amxx\tdebug\t; Respawn items after warm up GG\n\
\n\
; Configuration\n\
\n\
;pausecfg.amxx\t\t; allows to pause\n\
\n\
motd_mdl.amxx\n\
\n\
antisg.amxx\n\
server_info.amxx\n";

    #[test]
    fn groups_round_trip_preserves_bytes() {
        let doc = Document::parse(Dialect::Amxx, GROUPING_INI);
        assert_eq!(doc.to_bytes(), GROUPING_INI);
    }

    #[test]
    fn groups_partition_named_and_other() {
        let doc = Document::parse(Dialect::Amxx, GROUPING_INI);
        let groups = doc.groups();

        let titles: Vec<Option<&str>> = groups.iter().map(|g| g.title.as_deref()).collect();
        assert_eq!(
            titles,
            vec![
                Some("AMXBans 1.6 GM"),
                None,
                Some("Admin Base"),
                Some("Basic"),
                Some("Half-Life GunGame"),
                Some("Configuration"),
                None,
                None,
            ]
        );

        let keys: Vec<Vec<&str>> = groups
            .iter()
            .map(|g| g.entries.iter().map(|e| e.fields.key()).collect())
            .collect();
        assert_eq!(keys[0], ["amxbans_core.amxx", "amxbans_main.amxx"]);
        assert_eq!(keys[1], ["restrictnames.amxx"]);
        assert_eq!(keys[2], ["admin.amxx", "admin_sql.amxx"]);
        assert_eq!(keys[3], ["admincmd.amxx"]);
        assert_eq!(keys[5], ["pausecfg.amxx"]);
        assert_eq!(keys[6], ["motd_mdl.amxx"]);
        assert_eq!(keys[7], ["antisg.amxx", "server_info.amxx"]);
    }

    #[test]
    fn amxx_debug_and_comment_parsing() {
        assert_eq!(amxx_debug_comment(""), (false, None));
        assert_eq!(amxx_debug_comment("debug"), (true, None));
        assert_eq!(
            amxx_debug_comment("debug ; Respawn items after warm up GG"),
            (true, Some("Respawn items after warm up GG".to_string()))
        );
        assert_eq!(
            amxx_debug_comment("; admin base"),
            (false, Some("admin base".to_string()))
        );
        // A bare inline marker is an empty comment.
        assert_eq!(amxx_debug_comment(";"), (false, None));
        // `debug` counts as the flag only when it is the first token.
        assert_eq!(
            amxx_debug_comment("; enable debug"),
            (false, Some("enable debug".to_string()))
        );
    }

    #[test]
    fn amxx_debug_and_comment_from_document() {
        let doc = Document::parse(Dialect::Amxx, GROUPING_INI);
        let mut seen = std::collections::HashMap::new();
        for entry in doc.entries() {
            if let EntryFields::Amxx { file, rest } = &entry.fields {
                seen.insert(file.clone(), amxx_debug_comment(rest));
            }
        }
        assert_eq!(seen["gungame.amxx"], (true, None));
        assert_eq!(
            seen["gg_respawnItems.amxx"],
            (true, Some("Respawn items after warm up GG".to_string()))
        );
        assert_eq!(
            seen["admincmd.amxx"],
            (false, Some("basic admin console commands".to_string()))
        );
        assert_eq!(seen["admin.amxx"], (false, Some("admin base".to_string())));
    }

    #[test]
    fn amxx_meta_body_is_canonical() {
        assert_eq!(amxx_meta_body("x.amxx", false, None), "x.amxx");
        assert_eq!(amxx_meta_body("x.amxx", true, None), "x.amxx debug");
        assert_eq!(
            amxx_meta_body("x.amxx", true, Some("note")),
            "x.amxx debug ; note"
        );
        assert_eq!(amxx_meta_body("x.amxx", false, Some("  ")), "x.amxx");
    }
}
