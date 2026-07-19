//! POST /servers/{id}/{platform}/plugins — register an uploaded plugin file.

use std::collections::HashMap;

use crate::goldsrc::{self, Platform, ini, paths};
use crate::handlers::ctx::{self, ServerCtx};
use crate::host_api::HostApi;
use crate::http::{ApiError, ApiResult, json_response, parse_json_body};
use crate::model::{AddPluginRequest, AddPluginResponse};

pub fn handle<H: HostApi>(
    host: &mut H,
    params: &HashMap<String, String>,
    body: &[u8],
) -> ApiResult {
    let context = ServerCtx::resolve(host, params)?;
    let platform = ctx::parse_platform(params)?;
    let request: AddPluginRequest = parse_json_body(body)?;
    paths::sanitize_file_name(&request.file).map_err(ApiError::bad_request)?;

    match platform {
        Platform::Amxx => add_amxx(host, &context, &request),
        Platform::Metamod => add_metamod(host, &context, &request),
    }
}

fn add_amxx<H: HostApi>(
    host: &mut H,
    context: &ServerCtx,
    request: &AddPluginRequest,
) -> ApiResult {
    if !request.file.to_ascii_lowercase().ends_with(".amxx") {
        return Err(ApiError::unprocessable(
            "INVALID_FILE_TYPE",
            "AMX Mod X plugins must be compiled .amxx files",
        ));
    }

    let amxx_dir_abs = paths::join(&context.mod_abs, goldsrc::AMXX_DIR);
    if !host
        .stat(context.node_id, &amxx_dir_abs)?
        .is_some_and(|s| s.is_dir)
    {
        return Err(ApiError::conflict(
            "AMXX_NOT_INSTALLED",
            "AMX Mod X is not installed on this server",
        ));
    }

    let file_abs = paths::join(
        &paths::join(&context.mod_abs, goldsrc::AMXX_PLUGINS_DIR),
        &request.file,
    );
    if host.stat(context.node_id, &file_abs)?.is_none() {
        return Err(ApiError::unprocessable(
            "FILE_NOT_UPLOADED",
            format!(
                "{} was not found in {}; upload it first",
                request.file,
                goldsrc::AMXX_PLUGINS_DIR
            ),
        ));
    }

    let ini_abs = paths::join(&context.mod_abs, goldsrc::AMXX_PLUGINS_INI);
    register_entry(
        host,
        context,
        &ini_abs,
        ini::Dialect::Amxx,
        &request.file,
        &request.file,
        request.enable,
        request.force,
    )
}

fn add_metamod<H: HostApi>(
    host: &mut H,
    context: &ServerCtx,
    request: &AddPluginRequest,
) -> ApiResult {
    let platform_keyword = match request.file.rsplit('.').next() {
        Some(ext) if ext.eq_ignore_ascii_case("so") => "linux",
        Some(ext) if ext.eq_ignore_ascii_case("dll") => "win32",
        _ => {
            return Err(ApiError::unprocessable(
                "INVALID_FILE_TYPE",
                "Metamod plugins must be .so or .dll binaries",
            ));
        }
    };

    let rel = match &request.path {
        Some(path) => {
            paths::sanitize_rel_path(path).map_err(ApiError::bad_request)?;
            if !paths::file_name(path).eq_ignore_ascii_case(&request.file) {
                return Err(ApiError::bad_request(
                    "path must point at the uploaded file",
                ));
            }
            path.clone()
        }
        None => format!(
            "addons/{}/{}",
            paths::file_stem(&request.file),
            request.file
        ),
    };

    let liblist = super::read_liblist(host, context)?;
    let (mm_dir, installed) = super::metamod_dir(&liblist);
    if !installed {
        return Err(ApiError::conflict(
            "METAMOD_NOT_INSTALLED",
            "Metamod is not installed on this server",
        ));
    }

    let file_abs = paths::join(&context.mod_abs, &rel);
    if host.stat(context.node_id, &file_abs)?.is_none() {
        return Err(ApiError::unprocessable(
            "FILE_NOT_UPLOADED",
            format!("{rel} was not found; upload it first"),
        ));
    }

    let ini_abs = paths::join(
        &context.mod_abs,
        &super::platform_ini_rel(Platform::Metamod, &mm_dir),
    );
    let line = format!("{platform_keyword} {rel}");
    register_entry(
        host,
        context,
        &ini_abs,
        ini::Dialect::Metamod,
        &request.file,
        &line,
        request.enable,
        request.force,
    )
}

#[allow(clippy::too_many_arguments)]
fn register_entry<H: HostApi>(
    host: &mut H,
    context: &ServerCtx,
    ini_abs: &str,
    dialect: ini::Dialect,
    file: &str,
    line: &str,
    enable: bool,
    force: bool,
) -> ApiResult {
    let bytes = match host.stat(context.node_id, ini_abs)? {
        Some(_) => host.download(context.node_id, ini_abs)?,
        None => Vec::new(),
    };
    let mut doc = ini::Document::parse(dialect, &bytes);

    let replaced = if doc.find_entry(file).is_some() {
        if !force {
            return Err(ApiError::conflict(
                "ALREADY_REGISTERED",
                format!("{file} is already present in plugins.ini"),
            ));
        }
        // The uploaded file has already overwritten the binary; rewrite the
        // ini line in place so the load order is preserved.
        doc.replace_entry(file, line, enable).map_err(|_| {
            ApiError::internal(format!("failed to replace the {file} entry"))
        })?;
        true
    } else {
        doc.append_entry(line, enable);
        false
    };

    host.upload(
        context.node_id,
        ini_abs,
        &doc.to_bytes(),
        super::INI_FILE_PERMISSIONS,
    )?;

    Ok(json_response(
        if replaced { 200 } else { 201 },
        &AddPluginResponse {
            file: file.to_string(),
            enabled: enable,
            line: line.to_string(),
            replaced,
        },
    ))
}
