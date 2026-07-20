//! POST /servers/{id}/{platform}/sources/compile — compile a .sma source with
//! the server's own amxxpc and drop the result into the plugins directory.

use std::collections::HashMap;

use crate::goldsrc::{self, Platform, amxxpc, paths};
use crate::handlers::ctx::{self, ServerCtx};
use crate::host_api::HostApi;
use crate::http::{ApiError, ApiResult, json_response, parse_json_body};
use crate::model::{CompileDiagnostic, CompileRequest, CompileResponse};

pub fn handle<H: HostApi>(
    host: &mut H,
    params: &HashMap<String, String>,
    body: &[u8],
) -> ApiResult {
    let context = ServerCtx::resolve(host, params)?;
    let platform = ctx::parse_platform(params)?;
    if platform != Platform::Amxx {
        // Only AMXX has sources; metamod keeps the route shaped consistently.
        return Err(ApiError::not_found("NOT_FOUND", "unknown platform"));
    }
    let request: CompileRequest = parse_json_body(body)?;
    paths::sanitize_file_name(&request.file).map_err(ApiError::bad_request)?;
    if !request.file.to_ascii_lowercase().ends_with(".sma") {
        return Err(ApiError::unprocessable(
            "INVALID_FILE_TYPE",
            "only .sma source files can be compiled",
        ));
    }

    let node = host
        .get_node(context.node_id)?
        .ok_or_else(|| ApiError::not_found("NODE_NOT_FOUND", "node not found"))?;

    let scripting_abs = paths::join(&context.mod_abs, goldsrc::AMXX_SCRIPTING_DIR);
    let source_abs = paths::join(&scripting_abs, &request.file);
    if host
        .stat(context.node_id, &source_abs)?
        .is_none_or(|s| s.is_dir)
    {
        return Err(ApiError::not_found(
            "SOURCE_NOT_FOUND",
            format!("{} was not found in {}", request.file, goldsrc::AMXX_SCRIPTING_DIR),
        ));
    }

    let binary = amxxpc::compiler_binary(&node.os);
    let compiler_abs = paths::join(&scripting_abs, binary);
    if host
        .stat(context.node_id, &compiler_abs)?
        .is_none_or(|s| s.is_dir)
    {
        return Err(ApiError::unprocessable(
            "COMPILER_NOT_FOUND",
            format!(
                "{binary} was not found in {}; install the AMX Mod X compiler package",
                goldsrc::AMXX_SCRIPTING_DIR
            ),
        ));
    }

    let stem = paths::file_stem(&request.file);
    let plugins_abs = paths::join(&context.mod_abs, goldsrc::AMXX_PLUGINS_DIR);
    let amxx_name = format!("{stem}.amxx");
    let amxx_abs = paths::join(&plugins_abs, &amxx_name);

    let command = amxxpc::build_command(&scripting_abs, &plugins_abs, &request.file, stem, &node.os);
    host.log_info(&format!("compiling {}: {command}", request.file));
    let result = host.execute_command(context.node_id, &command, Some(&scripting_abs))?;

    // The exit code is the primary signal; the output file check guards
    // against wrappers that report success without producing a binary.
    let success = result.exit_code == 0
        && host
            .stat(context.node_id, &amxx_abs)?
            .is_some_and(|s| !s.is_dir);

    let diagnostics = amxxpc::parse_diagnostics(&result.output)
        .into_iter()
        .map(|diag| CompileDiagnostic {
            severity: diag.severity.as_str().to_string(),
            code: diag.code,
            line: diag.line,
            line_end: diag.line_end,
            message: diag.message,
        })
        .collect();

    Ok(json_response(
        200,
        &CompileResponse {
            file: request.file,
            success,
            exit_code: result.exit_code,
            output: result.output,
            diagnostics,
            amxx_file: success.then_some(amxx_name),
        },
    ))
}
