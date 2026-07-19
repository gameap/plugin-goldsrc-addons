//! DELETE /servers/{id}/{platform}/plugins — drop the ini entry and the file.

use std::collections::HashMap;

use gameap_plugin_sdk::proto::gameap::plugin as pb;

use crate::goldsrc::{self, Platform, ini, paths};
use crate::handlers::ctx::{self, ServerCtx};
use crate::host_api::HostApi;
use crate::http::{ApiError, ApiResult, json_response, parse_json_body};
use crate::model::{RemovePluginRequest, RemovePluginResponse};

pub fn handle<H: HostApi>(
    host: &mut H,
    params: &HashMap<String, String>,
    body: &[u8],
    query_params: &HashMap<String, pb::QueryParamValues>,
) -> ApiResult {
    let context = ServerCtx::resolve(host, params)?;
    let platform = ctx::parse_platform(params)?;

    // DELETE bodies are dropped by some proxies — accept ?file= as a fallback.
    let file = if body.is_empty() {
        query_params
            .get("file")
            .and_then(|values| values.values.first())
            .cloned()
            .ok_or_else(|| ApiError::bad_request("file is required (body or ?file=)"))?
    } else {
        parse_json_body::<RemovePluginRequest>(body)?.file
    };
    ctx::sanitize_file_key(&file)?;

    if platform == Platform::Metamod && goldsrc::is_amxx_loader_entry(&file) {
        return Err(ApiError::unprocessable(
            "SYSTEM_ENTRY",
            "the AMX Mod X loader entry is managed by the platform",
        ));
    }

    let ini_abs = super::require_platform_ini(host, &context, platform)?;
    let bytes = host.download(context.node_id, &ini_abs)?;
    let mut doc = ini::Document::parse(platform.dialect(), &bytes);

    let entry = doc.remove_entry(&file).map_err(|_| {
        ApiError::not_found(
            "PLUGIN_NOT_REGISTERED",
            format!("{file} is not present in plugins.ini"),
        )
    })?;

    host.upload(
        context.node_id,
        &ini_abs,
        &doc.to_bytes(),
        super::INI_FILE_PERMISSIONS,
    )?;

    let target_rel = match &entry.fields {
        ini::EntryFields::Metamod { path, .. } => paths::normalize_slashes(path),
        ini::EntryFields::Amxx { file, .. } => paths::join(goldsrc::AMXX_PLUGINS_DIR, file),
    };
    let target_abs = paths::join(&context.mod_abs, &target_rel);

    // Keep configs and the plugin directory; a failed file delete (already
    // gone, custom layout) degrades to file_deleted=false.
    let file_deleted = match host.remove(context.node_id, &target_abs, false) {
        Ok(()) => true,
        Err(err) => {
            host.log_error(&format!(
                "goldsrc-addons: failed to delete {target_abs}: {err:?}"
            ));
            false
        }
    };

    Ok(json_response(
        200,
        &RemovePluginResponse {
            file,
            entry_removed: true,
            file_deleted,
        },
    ))
}
