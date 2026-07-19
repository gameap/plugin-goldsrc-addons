//! Shared request context: server → game (engine gate) → node → mod dir.

use std::collections::HashMap;

use crate::goldsrc::{Platform, moddir, paths};
use crate::host_api::{HostApi, HostApiError};
use crate::http::ApiError;

const GOLDSOURCE_ENGINE: &str = "goldsource";
/// Upper bound of subdirectories probed for liblist.gam.
const MOD_DIR_SCAN_CAP: usize = 16;

pub struct ServerCtx {
    pub server_id: u64,
    pub node_id: u64,
    pub game_code: String,
    pub engine: String,
    /// Absolute server root on the node.
    pub root_abs: String,
    /// Mod folder name, e.g. "cstrike".
    pub mod_dir: String,
    /// Absolute mod folder path.
    pub mod_abs: String,
}

impl ServerCtx {
    pub fn resolve<H: HostApi>(
        host: &mut H,
        params: &HashMap<String, String>,
    ) -> Result<ServerCtx, ApiError> {
        let server_id: u64 = params
            .get("id")
            .and_then(|raw| raw.parse().ok())
            .ok_or_else(|| ApiError::bad_request("invalid server id"))?;

        let server = host
            .get_server(server_id)?
            .ok_or_else(|| ApiError::not_found("SERVER_NOT_FOUND", "server not found"))?;

        let game = host
            .get_game(&server.game_code)?
            .ok_or_else(|| ApiError::not_found("GAME_NOT_FOUND", "game not found"))?;
        if !game.engine.eq_ignore_ascii_case(GOLDSOURCE_ENGINE) {
            return Err(ApiError::unprocessable(
                "UNSUPPORTED_ENGINE",
                format!("server engine is {:?}, expected GoldSource", game.engine),
            ));
        }

        let node = host
            .get_node(server.node_id)?
            .ok_or_else(|| ApiError::not_found("NODE_NOT_FOUND", "node not found"))?;

        let root_abs = paths::join(&node.work_path, &server.dir);
        let mod_dir = resolve_mod_dir(host, node.id, &root_abs, &server.game_code)?
            .ok_or_else(|| {
                ApiError::unprocessable(
                    "MOD_DIR_NOT_FOUND",
                    "could not locate the mod directory (liblist.gam) inside the server directory",
                )
            })?;
        let mod_abs = paths::join(&root_abs, &mod_dir);

        Ok(ServerCtx {
            server_id,
            node_id: node.id,
            game_code: server.game_code,
            engine: game.engine,
            root_abs,
            mod_dir,
            mod_abs,
        })
    }

    /// Path relative to the server dir, for the frontend file-manager API.
    pub fn rel(&self, mod_relative: &str) -> String {
        paths::join(&self.mod_dir, mod_relative)
    }
}

fn resolve_mod_dir<H: HostApi>(
    host: &mut H,
    node_id: u64,
    root_abs: &str,
    game_code: &str,
) -> Result<Option<String>, HostApiError> {
    let hint = moddir::known_mod_dir(game_code);
    if let Some(dir) = hint
        && has_liblist(host, node_id, root_abs, dir)?
    {
        return Ok(Some(dir.to_string()));
    }

    let Some(entries) = host.read_dir(node_id, root_abs)? else {
        return Ok(None);
    };
    for entry in entries.iter().filter(|e| e.is_dir).take(MOD_DIR_SCAN_CAP) {
        if Some(entry.name.as_str()) == hint {
            continue;
        }
        if has_liblist(host, node_id, root_abs, &entry.name)? {
            return Ok(Some(entry.name.clone()));
        }
    }
    Ok(None)
}

fn has_liblist<H: HostApi>(
    host: &mut H,
    node_id: u64,
    root_abs: &str,
    dir: &str,
) -> Result<bool, HostApiError> {
    let liblist = paths::join(&paths::join(root_abs, dir), crate::goldsrc::LIBLIST_FILE);
    Ok(host.stat(node_id, &liblist)?.is_some_and(|s| !s.is_dir))
}

pub fn parse_platform(params: &HashMap<String, String>) -> Result<Platform, ApiError> {
    params
        .get("platform")
        .and_then(|raw| Platform::from_route_param(raw))
        .ok_or_else(|| ApiError::not_found("NOT_FOUND", "unknown platform"))
}

/// Validates a `file` request field: a bare name, or (metamod) an ini path.
pub fn sanitize_file_key(file: &str) -> Result<(), ApiError> {
    let result = if file.contains('/') {
        paths::sanitize_rel_path(file)
    } else {
        paths::sanitize_file_name(file)
    };
    result.map_err(ApiError::bad_request)
}
