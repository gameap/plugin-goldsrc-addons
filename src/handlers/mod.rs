pub mod add;
pub mod attributes;
pub mod ctx;
pub mod remove;
pub mod state;
pub mod toggle;

#[cfg(test)]
mod tests;

use crate::goldsrc::{self, Platform, liblist, paths};
use crate::host_api::HostApi;
use crate::http::ApiError;

use ctx::ServerCtx;

/// Reads liblist.gam of the mod; a missing file yields an empty Liblist.
fn read_liblist<H: HostApi>(host: &mut H, ctx: &ServerCtx) -> Result<liblist::Liblist, ApiError> {
    let liblist_abs = paths::join(&ctx.mod_abs, goldsrc::LIBLIST_FILE);
    if host.stat(ctx.node_id, &liblist_abs)?.is_none() {
        return Ok(liblist::Liblist::default());
    }
    let bytes = host.download(ctx.node_id, &liblist_abs)?;
    Ok(liblist::parse(&bytes))
}

/// Metamod addons dir (mod-relative) and whether liblist actually points at it.
fn metamod_dir(liblist: &liblist::Liblist) -> (String, bool) {
    match liblist::metamod_dir_from(liblist) {
        Some(dir) => (dir, true),
        None => (goldsrc::METAMOD_DIR_DEFAULT.to_string(), false),
    }
}

/// plugins.ini location (mod-relative) for a platform.
fn platform_ini_rel(platform: Platform, metamod_dir: &str) -> String {
    match platform {
        Platform::Metamod => paths::join(metamod_dir, "plugins.ini"),
        Platform::Amxx => goldsrc::AMXX_PLUGINS_INI.to_string(),
    }
}

/// Resolves the plugins.ini absolute path for mutations; 404 when the file
/// does not exist (platform not installed).
fn require_platform_ini<H: HostApi>(
    host: &mut H,
    ctx: &ServerCtx,
    platform: Platform,
) -> Result<String, ApiError> {
    let liblist = read_liblist(host, ctx)?;
    let (mm_dir, _) = metamod_dir(&liblist);
    let ini_abs = paths::join(&ctx.mod_abs, &platform_ini_rel(platform, &mm_dir));
    if host.stat(ctx.node_id, &ini_abs)?.is_none() {
        let name = match platform {
            Platform::Metamod => "Metamod",
            Platform::Amxx => "AMX Mod X",
        };
        return Err(ApiError::not_found(
            "PLUGINS_INI_NOT_FOUND",
            format!("plugins.ini not found — is {name} installed?"),
        ));
    }
    Ok(ini_abs)
}

const INI_FILE_PERMISSIONS: u32 = 0o644;
