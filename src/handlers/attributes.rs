//! POST /servers/{id}/{platform}/plugins/attributes — set an entry's debug
//! flag and inline comment, rewriting only that line.

use std::collections::HashMap;

use crate::goldsrc::{self, Platform, ini};
use crate::handlers::ctx::{self, ServerCtx};
use crate::host_api::HostApi;
use crate::http::{ApiError, ApiResult, json_response, parse_json_body};
use crate::model::{SetAttributesRequest, SetAttributesResponse};

const MAX_COMMENT_LEN: usize = 200;

pub fn handle<H: HostApi>(
    host: &mut H,
    params: &HashMap<String, String>,
    body: &[u8],
) -> ApiResult {
    let context = ServerCtx::resolve(host, params)?;
    let platform = ctx::parse_platform(params)?;
    let request: SetAttributesRequest = parse_json_body(body)?;
    ctx::sanitize_file_key(&request.file)?;

    if platform == Platform::Metamod && goldsrc::is_amxx_loader_entry(&request.file) {
        return Err(ApiError::unprocessable(
            "SYSTEM_ENTRY",
            "the AMX Mod X loader entry is managed by the platform",
        ));
    }

    // Reject/strip user input that could inject a new line into plugins.ini.
    let comment = normalize_comment(request.comment)?;

    let ini_abs = super::require_platform_ini(host, &context, platform)?;
    let bytes = host.download(context.node_id, &ini_abs)?;
    let mut doc = ini::Document::parse(platform.dialect(), &bytes);

    let index = doc.find_entry(&request.file).ok_or_else(|| {
        ApiError::not_found(
            "PLUGIN_NOT_REGISTERED",
            format!("{} is not present in plugins.ini", request.file),
        )
    })?;
    let Some(entry) = doc.entry_at(index) else {
        return Err(ApiError::internal("plugins.ini entry lookup failed"));
    };

    let enabled = entry.enabled;
    let (desired_debug, current_debug, current_comment, new_body) = match &entry.fields {
        ini::EntryFields::Amxx { file, rest } => {
            let (current_debug, current_comment) = ini::amxx_debug_comment(rest);
            let new_body = ini::amxx_meta_body(file, request.debug, comment.as_deref());
            (request.debug, current_debug, current_comment, new_body)
        }
        // Metamod has no debug flag; its "comment" is the free-text description
        // that trails `<platform> <path>`.
        ini::EntryFields::Metamod {
            platform: os,
            path,
            rest,
        } => {
            let current_comment = (!rest.is_empty()).then(|| rest.clone());
            let mut new_body = format!("{os} {path}");
            if let Some(comment) = &comment {
                new_body.push(' ');
                new_body.push_str(comment);
            }
            (false, false, current_comment, new_body)
        }
    };

    let changed = desired_debug != current_debug || comment != current_comment;
    if changed {
        doc.replace_entry(&request.file, &new_body, enabled)
            .map_err(|_| ApiError::internal("failed to rewrite the plugins.ini entry"))?;
        host.upload(
            context.node_id,
            &ini_abs,
            &doc.to_bytes(),
            super::INI_FILE_PERMISSIONS,
        )?;
    }

    Ok(json_response(
        200,
        &SetAttributesResponse {
            file: request.file,
            debug: desired_debug,
            comment,
            changed,
        },
    ))
}

/// Validates and normalizes an incoming comment: rejects control characters
/// (which would split the ini line), strips a leading marker the user may have
/// typed, trims, and enforces a length cap. Empty result becomes `None`.
fn normalize_comment(comment: Option<String>) -> Result<Option<String>, ApiError> {
    let Some(raw) = comment else {
        return Ok(None);
    };
    if raw.chars().any(char::is_control) {
        return Err(ApiError::bad_request(
            "comment must not contain control characters",
        ));
    }
    let text = raw
        .trim()
        .trim_start_matches([';', '#', '/'])
        .trim();
    if text.chars().count() > MAX_COMMENT_LEN {
        return Err(ApiError::bad_request("comment is too long"));
    }
    Ok((!text.is_empty()).then(|| text.to_string()))
}
