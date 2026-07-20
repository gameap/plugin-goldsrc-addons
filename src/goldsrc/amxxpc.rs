//! amxxpc (the AMX Mod X compiler) integration: command assembly and output
//! parsing.
//!
//! amxxpc is a wrapper around libpc300: it compiles to an intermediate `.amx`
//! and only writes the final `.amxx` after a successful compile + compression
//! step, removing the intermediate on failure. A failed compile therefore
//! leaves a previously compiled `.amxx` at the target path untouched, so it is
//! safe to point `-o` straight into the plugins directory.
//!
//! Diagnostic lines look like:
//!   `path/to/file.sma(123) : error 017: undefined symbol "foo"`
//!   `path/to/file.sma(10 -- 12) : warning 217: loose indentation`
//! Severities are `error`, `warning` and `fatal error`. The compiler reports
//! line numbers only, never columns.

/// The compiler executable name for the node's OS.
pub fn compiler_binary(os: &str) -> &'static str {
    if os.eq_ignore_ascii_case("windows") {
        "amxxpc.exe"
    } else {
        "amxxpc"
    }
}

/// Wraps a path in double quotes for the shell command line.
fn quote(arg: &str) -> String {
    format!("\"{arg}\"")
}

/// Builds the compile command. `work_dir` must be the scripting directory
/// (amxxpc dlopens `./amxxpc32.so` relative to its CWD), so every argument is
/// absolute.
pub fn build_command(
    scripting_abs: &str,
    plugins_abs: &str,
    sma_file: &str,
    stem: &str,
    os: &str,
) -> String {
    let compiler = paths_join(scripting_abs, compiler_binary(os));
    let source = paths_join(scripting_abs, sma_file);
    let include = paths_join(scripting_abs, "include");
    let output = paths_join(plugins_abs, &format!("{stem}.amxx"));
    format!(
        "{} {} -i{} -o{}",
        quote(&compiler),
        quote(&source),
        quote(&include),
        quote(&output)
    )
}

fn paths_join(base: &str, rel: &str) -> String {
    format!("{}/{}", base.trim_end_matches('/'), rel)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: u32,
    pub line: u32,
    /// End line of a `(10 -- 12)` range, when reported.
    pub line_end: Option<u32>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    FatalError,
    Warning,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::FatalError => "fatal error",
            Severity::Warning => "warning",
        }
    }
}

/// Parses amxxpc output into structured diagnostics; unrecognized lines are
/// skipped (banner, sizes, the `N Error(s).` summary).
pub fn parse_diagnostics(output: &str) -> Vec<Diagnostic> {
    output.lines().filter_map(parse_diagnostic_line).collect()
}

fn parse_diagnostic_line(line: &str) -> Option<Diagnostic> {
    // `<file>(<line>[ -- <end>]) : <severity> <code>: <message>`
    let open = line.rfind('(')?;
    let close = line[open..].find(')')? + open;
    let (line_no, line_end) = parse_line_range(&line[open + 1..close])?;
    let rest = line[close + 1..].strip_prefix(" : ")?;
    let (severity, rest) = split_severity(rest)?;
    let (code, message) = rest.split_once(": ")?;
    Some(Diagnostic {
        severity,
        code: code.trim().parse().ok()?,
        line: line_no,
        line_end,
        message: message.to_string(),
    })
}

fn parse_line_range(raw: &str) -> Option<(u32, Option<u32>)> {
    match raw.split_once("--") {
        Some((first, last)) => Some((
            first.trim().parse().ok()?,
            Some(last.trim().parse().ok()?),
        )),
        None => Some((raw.trim().parse().ok()?, None)),
    }
}

fn split_severity(rest: &str) -> Option<(Severity, &str)> {
    for (prefix, severity) in [
        ("fatal error ", Severity::FatalError),
        ("error ", Severity::Error),
        ("warning ", Severity::Warning),
    ] {
        if let Some(tail) = rest.strip_prefix(prefix) {
            return Some((severity, tail));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiler_binary_per_os() {
        assert_eq!(compiler_binary("windows"), "amxxpc.exe");
        assert_eq!(compiler_binary("Windows"), "amxxpc.exe");
        assert_eq!(compiler_binary("linux"), "amxxpc");
        assert_eq!(compiler_binary("darwin"), "amxxpc");
    }

    #[test]
    fn builds_command() {
        let cmd = build_command(
            "/srv/cs/cstrike/addons/amxmodx/scripting",
            "/srv/cs/cstrike/addons/amxmodx/plugins",
            "gungame.sma",
            "gungame",
            "linux",
        );
        assert_eq!(
            cmd,
            "\"/srv/cs/cstrike/addons/amxmodx/scripting/amxxpc\" \
             \"/srv/cs/cstrike/addons/amxmodx/scripting/gungame.sma\" \
             -i\"/srv/cs/cstrike/addons/amxmodx/scripting/include\" \
             -o\"/srv/cs/cstrike/addons/amxmodx/plugins/gungame.amxx\""
        );
    }

    #[test]
    fn parses_errors_warnings_and_ranges() {
        let output = "\
AMX Mod X Compiler 1.9.0.5294
Copyright (c) 1997-2006 ITB CompuPhase
Copyright (c) 2004-2013 AMX Mod X Team

/srv/cs/gungame.sma(12) : error 017: undefined symbol \"gg_enable\"
/srv/cs/gungame.sma(30 -- 32) : warning 217: loose indentation
/srv/cs/include/amxmodx.inc(45) : fatal error 100: cannot read from file
2 Errors.
Could not locate output file /srv/cs/gungame.amx (compile failed).
";
        let diags = parse_diagnostics(output);
        assert_eq!(
            diags,
            vec![
                Diagnostic {
                    severity: Severity::Error,
                    code: 17,
                    line: 12,
                    line_end: None,
                    message: "undefined symbol \"gg_enable\"".into(),
                },
                Diagnostic {
                    severity: Severity::Warning,
                    code: 217,
                    line: 30,
                    line_end: Some(32),
                    message: "loose indentation".into(),
                },
                Diagnostic {
                    severity: Severity::FatalError,
                    code: 100,
                    line: 45,
                    line_end: None,
                    message: "cannot read from file".into(),
                },
            ]
        );
    }

    #[test]
    fn skips_noise() {
        assert!(parse_diagnostics("Done.\nHeader size: 1234\n").is_empty());
        assert!(parse_diagnostics("").is_empty());
    }
}
