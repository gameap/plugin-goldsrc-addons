//! POST /servers/{id}/{platform}/plugins/toggle — comment/uncomment an entry.

use std::collections::HashMap;

use crate::goldsrc::{self, Platform, ini};
use crate::handlers::ctx::{self, ServerCtx};
use crate::host_api::HostApi;
use crate::http::{ApiError, ApiResult, json_response, parse_json_body};
use crate::model::{ToggleRequest, ToggleResponse};

pub fn handle<H: HostApi>(
    host: &mut H,
    params: &HashMap<String, String>,
    body: &[u8],
) -> ApiResult {
    let context = ServerCtx::resolve(host, params)?;
    let platform = ctx::parse_platform(params)?;
    let request: ToggleRequest = parse_json_body(body)?;
    ctx::sanitize_file_key(&request.file)?;

    if platform == Platform::Metamod && goldsrc::is_amxx_loader_entry(&request.file) {
        return Err(ApiError::unprocessable(
            "SYSTEM_ENTRY",
            "the AMX Mod X loader entry is managed by the platform",
        ));
    }

    let ini_abs = super::require_platform_ini(host, &context, platform)?;
    let bytes = host.download(context.node_id, &ini_abs)?;
    let mut doc = ini::Document::parse(platform.dialect(), &bytes);

    let changed = doc
        .set_enabled(&request.file, request.enabled)
        .map_err(|_| {
            ApiError::not_found(
                "PLUGIN_NOT_REGISTERED",
                format!("{} is not present in plugins.ini", request.file),
            )
        })?;

    if changed {
        host.upload(
            context.node_id,
            &ini_abs,
            &doc.to_bytes(),
            super::INI_FILE_PERMISSIONS,
        )?;
    }

    Ok(json_response(
        200,
        &ToggleResponse {
            file: request.file,
            enabled: request.enabled,
            changed,
        },
    ))
}
