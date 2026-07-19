//! GET /servers/{id}/state — assembles the Metamod/AMXX picture of a server.

use std::collections::{HashMap, HashSet};

use crate::goldsrc::{self, ini, paths};
use crate::handlers::ctx::ServerCtx;
use crate::host_api::{HostApi, HostApiError};
use crate::http::{ApiResult, json_response};
use crate::model::{
    AmxxPluginEntry, AmxxState, MetamodPluginEntry, MetamodState, StatePaths, StateResponse,
};

pub fn handle<H: HostApi>(host: &mut H, params: &HashMap<String, String>) -> ApiResult {
    let ctx = ServerCtx::resolve(host, params)?;

    let liblist = super::read_liblist(host, &ctx)?;
    let (mm_dir, mm_installed) = super::metamod_dir(&liblist);
    let mm_dir_present = host
        .stat(ctx.node_id, &paths::join(&ctx.mod_abs, &mm_dir))?
        .is_some_and(|s| s.is_dir);

    let mm_ini_rel = super::platform_ini_rel(goldsrc::Platform::Metamod, &mm_dir);
    let mm_doc = read_ini_document(host, &ctx, &mm_ini_rel, ini::Dialect::Metamod)?;

    let amxx_installed = host
        .stat(ctx.node_id, &paths::join(&ctx.mod_abs, goldsrc::AMXX_DIR))?
        .is_some_and(|s| s.is_dir);
    let amxx_doc = read_ini_document(host, &ctx, goldsrc::AMXX_PLUGINS_INI, ini::Dialect::Amxx)?;

    let mut dir_cache: HashMap<String, Option<HashSet<String>>> = HashMap::new();

    let mut metamod_plugins = Vec::new();
    let mut registered_in_metamod = false;
    if let Some(doc) = &mm_doc {
        let mut next_named = 0u32;
        for group in doc.groups() {
            let (group_index, group_title) = assign_group(&group.title, &mut next_named);
            for entry in group.entries {
                let ini::EntryFields::Metamod {
                    platform,
                    path,
                    rest,
                } = &entry.fields
                else {
                    continue;
                };
                let normalized = paths::normalize_slashes(path);
                let system = goldsrc::is_amxx_loader_entry(&normalized);
                if system && entry.enabled {
                    registered_in_metamod = true;
                }
                let missing = !file_exists_cached(host, &ctx, &mut dir_cache, &normalized)?;
                metamod_plugins.push(MetamodPluginEntry {
                    platform: platform.clone(),
                    path: path.clone(),
                    file: paths::file_name(&normalized).to_string(),
                    description: (!rest.is_empty()).then(|| rest.clone()),
                    enabled: entry.enabled,
                    missing,
                    system,
                    group_index,
                    group_title: group_title.clone(),
                });
            }
        }
    }

    let plugins_files = list_dir_names(host, &ctx, &mut dir_cache, goldsrc::AMXX_PLUGINS_DIR)?;
    let configs_files = list_dir_names(host, &ctx, &mut dir_cache, goldsrc::AMXX_CONFIGS_DIR)?;
    let configs_lower: HashMap<String, String> = configs_files
        .iter()
        .map(|name| (name.to_ascii_lowercase(), name.clone()))
        .collect();

    let mut amxx_plugins = Vec::new();
    if let Some(doc) = &amxx_doc {
        let mut next_named = 0u32;
        for group in doc.groups() {
            let (group_index, group_title) = assign_group(&group.title, &mut next_named);
            for entry in group.entries {
                let ini::EntryFields::Amxx { file, rest } = &entry.fields else {
                    continue;
                };
                let (debug, comment) = ini::amxx_debug_comment(rest);
                let missing = !plugins_files.contains(file);
                let config_path = config_candidates(file)
                    .into_iter()
                    .find_map(|candidate| configs_lower.get(&candidate))
                    .map(|original| ctx.rel(&paths::join(goldsrc::AMXX_CONFIGS_DIR, original)));
                amxx_plugins.push(AmxxPluginEntry {
                    file: file.clone(),
                    debug,
                    comment,
                    enabled: entry.enabled,
                    missing,
                    has_config: config_path.is_some(),
                    config_path,
                    group_index,
                    group_title: group_title.clone(),
                });
            }
        }
    }

    let response = StateResponse {
        server_id: ctx.server_id,
        game_code: ctx.game_code.clone(),
        engine: ctx.engine.clone(),
        mod_dir: ctx.mod_dir.clone(),
        paths: StatePaths {
            liblist: ctx.rel(goldsrc::LIBLIST_FILE),
            metamod_dir: ctx.rel(&mm_dir),
            metamod_plugins_ini: ctx.rel(&mm_ini_rel),
            amxx_dir: ctx.rel(goldsrc::AMXX_DIR),
            amxx_plugins_ini: ctx.rel(goldsrc::AMXX_PLUGINS_INI),
            amxx_plugins_dir: ctx.rel(goldsrc::AMXX_PLUGINS_DIR),
            amxx_configs_dir: ctx.rel(goldsrc::AMXX_CONFIGS_DIR),
        },
        metamod: MetamodState {
            installed: mm_installed,
            dir_present: mm_dir_present,
            plugins_ini_exists: mm_doc.is_some(),
            plugins: metamod_plugins,
        },
        amxx: AmxxState {
            installed: amxx_installed,
            registered_in_metamod,
            plugins_ini_exists: amxx_doc.is_some(),
            plugins: amxx_plugins,
        },
    };

    Ok(json_response(200, &response))
}

/// Every unnamed group collapses into a single "Other" bucket, sorted last.
const OTHER_GROUP: u32 = u32::MAX;

/// Display index/title for a parsed group: named groups get sequential indices
/// by appearance, unnamed ones share the trailing "Other" bucket.
fn assign_group(title: &Option<String>, next_named: &mut u32) -> (u32, Option<String>) {
    match title {
        Some(title) => {
            let index = *next_named;
            *next_named += 1;
            (index, Some(title.clone()))
        }
        None => (OTHER_GROUP, None),
    }
}

/// Lower-cased config file names an AMXX plugin is looked up by:
/// `<stem>.cfg`, `<stem>.ini` and known exceptions of popular plugins.
fn config_candidates(file: &str) -> Vec<String> {
    let stem = paths::file_stem(file).to_ascii_lowercase();
    let mut candidates = vec![format!("{stem}.cfg"), format!("{stem}.ini")];
    match stem.as_str() {
        "statsx" => candidates.push("stats.ini".into()),
        "csdm_main" => candidates.push("csdm.cfg".into()),
        _ => {}
    }
    candidates
}

fn read_ini_document<H: HostApi>(
    host: &mut H,
    ctx: &ServerCtx,
    ini_rel: &str,
    dialect: ini::Dialect,
) -> Result<Option<ini::Document>, HostApiError> {
    let ini_abs = paths::join(&ctx.mod_abs, ini_rel);
    if host.stat(ctx.node_id, &ini_abs)?.is_none() {
        return Ok(None);
    }
    let bytes = host.download(ctx.node_id, &ini_abs)?;
    Ok(Some(ini::Document::parse(dialect, &bytes)))
}

/// File names directly inside a mod-relative directory (empty when missing).
fn list_dir_names<H: HostApi>(
    host: &mut H,
    ctx: &ServerCtx,
    cache: &mut HashMap<String, Option<HashSet<String>>>,
    dir_rel: &str,
) -> Result<HashSet<String>, HostApiError> {
    if let Some(cached) = cache.get(dir_rel) {
        return Ok(cached.clone().unwrap_or_default());
    }
    let dir_abs = paths::join(&ctx.mod_abs, dir_rel);
    let names = host.read_dir(ctx.node_id, &dir_abs)?.map(|entries| {
        entries
            .into_iter()
            .filter(|e| !e.is_dir)
            .map(|e| e.name)
            .collect::<HashSet<_>>()
    });
    cache.insert(dir_rel.to_string(), names.clone());
    Ok(names.unwrap_or_default())
}

/// Existence check for a mod-relative file path via its parent dir listing.
fn file_exists_cached<H: HostApi>(
    host: &mut H,
    ctx: &ServerCtx,
    cache: &mut HashMap<String, Option<HashSet<String>>>,
    file_rel: &str,
) -> Result<bool, HostApiError> {
    let (parent, name) = match file_rel.rfind('/') {
        Some(idx) => (&file_rel[..idx], &file_rel[idx + 1..]),
        None => ("", file_rel),
    };
    let names = list_dir_names(host, ctx, cache, parent)?;
    Ok(names.contains(name))
}
