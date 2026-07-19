//! Handler tests driven through `router::dispatch` against `MockHost`.

use gameap_plugin_sdk::proto::gameap::plugin as pb;
use serde_json::Value;

use crate::host_api::mock::MockHost;
use crate::router::dispatch;

const MOD: &str = MockHost::MOD_ABS;

fn request(method: &str, path: &str, body: &[u8]) -> pb::HttpRequest {
    pb::HttpRequest {
        method: method.into(),
        path: path.into(),
        body: body.to_vec(),
        ..Default::default()
    }
}

fn json(resp: &pb::HttpResponse) -> Value {
    serde_json::from_slice(&resp.body).expect("json body")
}

fn full_setup() -> MockHost {
    let mut host = MockHost::goldsource();
    host.add_file(
        &format!("{MOD}/liblist.gam"),
        b"game \"Counter-Strike\"\ngamedll \"dlls\\mp.dll\"\ngamedll_linux \"addons/metamod/dlls/metamod_i386.so\"\n",
    );
    host.add_file(
        &format!("{MOD}/addons/metamod/plugins.ini"),
        b"linux addons/amxmodx/dlls/amxmodx_mm_i386.so\nlinux addons/reunion/reunion_mm_i386.so\n;linux addons/whblocker/whblocker_mm_i386.so\n",
    );
    host.add_file(&format!("{MOD}/addons/amxmodx/dlls/amxmodx_mm_i386.so"), b"elf");
    host.add_file(&format!("{MOD}/addons/reunion/reunion_mm_i386.so"), b"elf");
    host.add_file(
        &format!("{MOD}/addons/amxmodx/configs/plugins.ini"),
        b"; AMX Mod X plugins\nadmin.amxx\nstatsx.amxx\n;parachute.amxx\n",
    );
    host.add_file(&format!("{MOD}/addons/amxmodx/plugins/admin.amxx"), b"amxx");
    host.add_file(&format!("{MOD}/addons/amxmodx/plugins/statsx.amxx"), b"amxx");
    host.add_file(&format!("{MOD}/addons/amxmodx/plugins/parachute.amxx"), b"amxx");
    host.add_file(&format!("{MOD}/addons/amxmodx/configs/stats.ini"), b"cfg");
    host
}

#[test]
fn state_full_setup() {
    let mut host = full_setup();
    let resp = dispatch(&mut host, &request("GET", "/servers/3/state", b""));
    assert_eq!(resp.status_code, 200, "{:?}", String::from_utf8_lossy(&resp.body));
    let body = json(&resp);

    assert_eq!(body["mod_dir"], "cstrike");
    assert_eq!(body["engine"], "GoldSource");
    assert_eq!(body["paths"]["amxx_plugins_ini"], "cstrike/addons/amxmodx/configs/plugins.ini");
    assert_eq!(body["paths"]["metamod_plugins_ini"], "cstrike/addons/metamod/plugins.ini");

    assert_eq!(body["metamod"]["installed"], true);
    assert_eq!(body["metamod"]["dir_present"], true);
    let mm_plugins = body["metamod"]["plugins"].as_array().expect("array");
    assert_eq!(mm_plugins.len(), 3);
    assert_eq!(mm_plugins[0]["system"], true);
    assert_eq!(mm_plugins[1]["file"], "reunion_mm_i386.so");
    assert_eq!(mm_plugins[1]["missing"], false);
    assert_eq!(mm_plugins[2]["enabled"], false);
    assert_eq!(mm_plugins[2]["missing"], true);

    assert_eq!(body["amxx"]["installed"], true);
    assert_eq!(body["amxx"]["registered_in_metamod"], true);
    let amxx_plugins = body["amxx"]["plugins"].as_array().expect("array");
    assert_eq!(amxx_plugins.len(), 3);
    assert_eq!(amxx_plugins[0]["file"], "admin.amxx");
    assert_eq!(amxx_plugins[0]["has_config"], false);
    assert_eq!(amxx_plugins[1]["file"], "statsx.amxx");
    assert_eq!(amxx_plugins[1]["has_config"], true);
    assert_eq!(
        amxx_plugins[1]["config_path"],
        "cstrike/addons/amxmodx/configs/stats.ini"
    );
    assert_eq!(amxx_plugins[2]["enabled"], false);
}

#[test]
fn state_clean_server() {
    let mut host = MockHost::goldsource();
    host.add_file(
        &format!("{MOD}/liblist.gam"),
        b"game \"Counter-Strike\"\ngamedll_linux \"dlls/cs.so\"\n",
    );
    let resp = dispatch(&mut host, &request("GET", "/servers/3/state", b""));
    assert_eq!(resp.status_code, 200);
    let body = json(&resp);
    assert_eq!(body["metamod"]["installed"], false);
    assert_eq!(body["metamod"]["plugins_ini_exists"], false);
    assert_eq!(body["amxx"]["installed"], false);
}

#[test]
fn state_rejects_non_goldsource() {
    let mut host = full_setup();
    if let Some(game) = host.games.get_mut("cstrike") {
        game.engine = "Source".into();
    }
    let resp = dispatch(&mut host, &request("GET", "/servers/3/state", b""));
    assert_eq!(resp.status_code, 422);
    assert_eq!(json(&resp)["code"], "UNSUPPORTED_ENGINE");
}

#[test]
fn state_unknown_server() {
    let mut host = full_setup();
    let resp = dispatch(&mut host, &request("GET", "/servers/99/state", b""));
    assert_eq!(resp.status_code, 404);
    assert_eq!(json(&resp)["code"], "SERVER_NOT_FOUND");
}

#[test]
fn toggle_amxx_plugin() {
    let mut host = full_setup();
    let resp = dispatch(
        &mut host,
        &request(
            "POST",
            "/servers/3/amxx/plugins/toggle",
            br#"{"file":"statsx.amxx","enabled":false}"#,
        ),
    );
    assert_eq!(resp.status_code, 200);
    assert_eq!(json(&resp)["changed"], true);
    let ini = host
        .file(&format!("{MOD}/addons/amxmodx/configs/plugins.ini"))
        .expect("ini exists");
    assert_eq!(ini, b"; AMX Mod X plugins\nadmin.amxx\n;statsx.amxx\n;parachute.amxx\n");

    // Idempotent second call.
    let resp = dispatch(
        &mut host,
        &request(
            "POST",
            "/servers/3/amxx/plugins/toggle",
            br#"{"file":"statsx.amxx","enabled":false}"#,
        ),
    );
    assert_eq!(json(&resp)["changed"], false);
}

#[test]
fn toggle_enables_commented_entry() {
    let mut host = full_setup();
    let resp = dispatch(
        &mut host,
        &request(
            "POST",
            "/servers/3/metamod/plugins/toggle",
            br#"{"file":"whblocker_mm_i386.so","enabled":true}"#,
        ),
    );
    assert_eq!(resp.status_code, 200);
    let ini = host
        .file(&format!("{MOD}/addons/metamod/plugins.ini"))
        .expect("ini exists");
    assert!(
        String::from_utf8_lossy(ini).contains("\nlinux addons/whblocker/whblocker_mm_i386.so\n")
    );
}

#[test]
fn toggle_protects_amxx_loader_entry() {
    let mut host = full_setup();
    let resp = dispatch(
        &mut host,
        &request(
            "POST",
            "/servers/3/metamod/plugins/toggle",
            br#"{"file":"amxmodx_mm_i386.so","enabled":false}"#,
        ),
    );
    assert_eq!(resp.status_code, 422);
    assert_eq!(json(&resp)["code"], "SYSTEM_ENTRY");
}

#[test]
fn toggle_unknown_plugin() {
    let mut host = full_setup();
    let resp = dispatch(
        &mut host,
        &request(
            "POST",
            "/servers/3/amxx/plugins/toggle",
            br#"{"file":"nope.amxx","enabled":true}"#,
        ),
    );
    assert_eq!(resp.status_code, 404);
    assert_eq!(json(&resp)["code"], "PLUGIN_NOT_REGISTERED");
}

#[test]
fn add_amxx_plugin() {
    let mut host = full_setup();

    // Not uploaded yet.
    let resp = dispatch(
        &mut host,
        &request(
            "POST",
            "/servers/3/amxx/plugins",
            br#"{"file":"galileo.amxx"}"#,
        ),
    );
    assert_eq!(resp.status_code, 422);
    assert_eq!(json(&resp)["code"], "FILE_NOT_UPLOADED");

    host.add_file(&format!("{MOD}/addons/amxmodx/plugins/galileo.amxx"), b"amxx");
    let resp = dispatch(
        &mut host,
        &request(
            "POST",
            "/servers/3/amxx/plugins",
            br#"{"file":"galileo.amxx"}"#,
        ),
    );
    assert_eq!(resp.status_code, 201, "{}", String::from_utf8_lossy(&resp.body));
    let ini = host
        .file(&format!("{MOD}/addons/amxmodx/configs/plugins.ini"))
        .expect("ini exists");
    assert!(String::from_utf8_lossy(ini).ends_with("galileo.amxx\n"));

    // Duplicate.
    let resp = dispatch(
        &mut host,
        &request(
            "POST",
            "/servers/3/amxx/plugins",
            br#"{"file":"galileo.amxx"}"#,
        ),
    );
    assert_eq!(resp.status_code, 409);
    assert_eq!(json(&resp)["code"], "ALREADY_REGISTERED");
}

#[test]
fn add_amxx_force_overwrites_existing() {
    let mut host = full_setup();

    // Without force the duplicate is rejected.
    let resp = dispatch(
        &mut host,
        &request(
            "POST",
            "/servers/3/amxx/plugins",
            br#"{"file":"statsx.amxx"}"#,
        ),
    );
    assert_eq!(resp.status_code, 409);

    // Force keeps the line position and applies the requested enabled state.
    let resp = dispatch(
        &mut host,
        &request(
            "POST",
            "/servers/3/amxx/plugins",
            br#"{"file":"statsx.amxx","enable":false,"force":true}"#,
        ),
    );
    assert_eq!(resp.status_code, 200, "{}", String::from_utf8_lossy(&resp.body));
    assert_eq!(json(&resp)["replaced"], true);
    let ini = host
        .file(&format!("{MOD}/addons/amxmodx/configs/plugins.ini"))
        .expect("ini exists");
    assert_eq!(ini, b"; AMX Mod X plugins\nadmin.amxx\n;statsx.amxx\n;parachute.amxx\n");
}

#[test]
fn add_metamod_force_replaces_line_in_place() {
    let mut host = full_setup();
    host.add_file(&format!("{MOD}/addons/reunion/reunion_mm.dll"), b"pe");
    let resp = dispatch(
        &mut host,
        &request(
            "POST",
            "/servers/3/metamod/plugins",
            br#"{"file":"reunion_mm.dll","path":"addons/reunion/reunion_mm.dll","force":true}"#,
        ),
    );
    // A different file name is not a duplicate of reunion_mm_i386.so — appended.
    assert_eq!(resp.status_code, 201);
    assert_eq!(json(&resp)["replaced"], false);

    // Same file name with a new path replaces the line without moving it.
    host.add_file(&format!("{MOD}/addons/reunion2/reunion_mm_i386.so"), b"elf");
    let resp = dispatch(
        &mut host,
        &request(
            "POST",
            "/servers/3/metamod/plugins",
            br#"{"file":"reunion_mm_i386.so","path":"addons/reunion2/reunion_mm_i386.so","force":true}"#,
        ),
    );
    assert_eq!(resp.status_code, 200, "{}", String::from_utf8_lossy(&resp.body));
    let ini = String::from_utf8_lossy(
        host.file(&format!("{MOD}/addons/metamod/plugins.ini"))
            .expect("ini exists"),
    )
    .into_owned();
    let lines: Vec<&str> = ini.lines().collect();
    assert_eq!(lines[1], "linux addons/reunion2/reunion_mm_i386.so");
    assert_eq!(lines[2], ";linux addons/whblocker/whblocker_mm_i386.so");
}

#[test]
fn add_amxx_rejects_sources() {
    let mut host = full_setup();
    let resp = dispatch(
        &mut host,
        &request("POST", "/servers/3/amxx/plugins", br#"{"file":"bhop.sma"}"#),
    );
    assert_eq!(resp.status_code, 422);
    assert_eq!(json(&resp)["code"], "INVALID_FILE_TYPE");
}

#[test]
fn add_metamod_plugin_disabled() {
    let mut host = full_setup();
    host.add_file(&format!("{MOD}/addons/podbot/podbot_mm_i386.so"), b"elf");
    let resp = dispatch(
        &mut host,
        &request(
            "POST",
            "/servers/3/metamod/plugins",
            br#"{"file":"podbot_mm_i386.so","enable":false,"path":"addons/podbot/podbot_mm_i386.so"}"#,
        ),
    );
    assert_eq!(resp.status_code, 201, "{}", String::from_utf8_lossy(&resp.body));
    assert_eq!(json(&resp)["line"], "linux addons/podbot/podbot_mm_i386.so");
    let ini = host
        .file(&format!("{MOD}/addons/metamod/plugins.ini"))
        .expect("ini exists");
    assert!(String::from_utf8_lossy(ini).ends_with(";linux addons/podbot/podbot_mm_i386.so\n"));
}

#[test]
fn add_metamod_default_path() {
    let mut host = full_setup();
    host.add_file(&format!("{MOD}/addons/whb/whb.so"), b"elf");
    let resp = dispatch(
        &mut host,
        &request("POST", "/servers/3/metamod/plugins", br#"{"file":"whb.so"}"#),
    );
    assert_eq!(resp.status_code, 201, "{}", String::from_utf8_lossy(&resp.body));
    assert_eq!(json(&resp)["line"], "linux addons/whb/whb.so");
}

#[test]
fn add_metamod_requires_metamod() {
    let mut host = MockHost::goldsource();
    host.add_file(
        &format!("{MOD}/liblist.gam"),
        b"gamedll_linux \"dlls/cs.so\"\n",
    );
    host.add_file(&format!("{MOD}/addons/whb/whb.so"), b"elf");
    let resp = dispatch(
        &mut host,
        &request("POST", "/servers/3/metamod/plugins", br#"{"file":"whb.so"}"#),
    );
    assert_eq!(resp.status_code, 409);
    assert_eq!(json(&resp)["code"], "METAMOD_NOT_INSTALLED");
}

#[test]
fn remove_amxx_plugin_via_body() {
    let mut host = full_setup();
    let resp = dispatch(
        &mut host,
        &request(
            "DELETE",
            "/servers/3/amxx/plugins",
            br#"{"file":"statsx.amxx"}"#,
        ),
    );
    assert_eq!(resp.status_code, 200);
    let body = json(&resp);
    assert_eq!(body["entry_removed"], true);
    assert_eq!(body["file_deleted"], true);
    let ini = host
        .file(&format!("{MOD}/addons/amxmodx/configs/plugins.ini"))
        .expect("ini exists");
    assert!(!String::from_utf8_lossy(ini).contains("statsx.amxx"));
    assert!(host.file(&format!("{MOD}/addons/amxmodx/plugins/statsx.amxx")).is_none());
    // Config stays.
    assert!(host.file(&format!("{MOD}/addons/amxmodx/configs/stats.ini")).is_some());
}

#[test]
fn remove_metamod_plugin_via_query() {
    let mut host = full_setup();
    let mut req = request("DELETE", "/servers/3/metamod/plugins", b"");
    req.query_params.insert(
        "file".into(),
        pb::QueryParamValues {
            values: vec!["reunion_mm_i386.so".into()],
        },
    );
    let resp = dispatch(&mut host, &req);
    assert_eq!(resp.status_code, 200, "{}", String::from_utf8_lossy(&resp.body));
    assert!(host.file(&format!("{MOD}/addons/reunion/reunion_mm_i386.so")).is_none());
    let ini = host
        .file(&format!("{MOD}/addons/metamod/plugins.ini"))
        .expect("ini exists");
    assert!(!String::from_utf8_lossy(ini).contains("reunion"));
}

#[test]
fn remove_missing_file_degrades() {
    let mut host = full_setup();
    host.files.remove(&format!("{MOD}/addons/amxmodx/plugins/statsx.amxx"));
    let resp = dispatch(
        &mut host,
        &request(
            "DELETE",
            "/servers/3/amxx/plugins",
            br#"{"file":"statsx.amxx"}"#,
        ),
    );
    assert_eq!(resp.status_code, 200);
    let body = json(&resp);
    assert_eq!(body["entry_removed"], true);
    assert_eq!(body["file_deleted"], false);
}

#[test]
fn unknown_route_is_json_404() {
    let mut host = full_setup();
    let resp = dispatch(&mut host, &request("GET", "/nope", b""));
    assert_eq!(resp.status_code, 404);
    assert_eq!(json(&resp)["code"], "NOT_FOUND");
}

#[test]
fn traversal_is_rejected() {
    let mut host = full_setup();
    let resp = dispatch(
        &mut host,
        &request(
            "POST",
            "/servers/3/amxx/plugins/toggle",
            br#"{"file":"../../liblist.gam","enabled":false}"#,
        ),
    );
    assert_eq!(resp.status_code, 400);
}

#[test]
fn state_reports_groups_debug_and_comment() {
    let mut host = full_setup();
    host.add_file(
        &format!("{MOD}/addons/amxmodx/configs/plugins.ini"),
        b"; Basic\nadmin.amxx\t; admin base\nstatsx.amxx\tdebug\t; stats\n\n; Fun\n\n;parachute.amxx\tdebug\n\nlone.amxx\n",
    );
    host.add_file(&format!("{MOD}/addons/amxmodx/plugins/lone.amxx"), b"amxx");

    let resp = dispatch(&mut host, &request("GET", "/servers/3/state", b""));
    assert_eq!(resp.status_code, 200, "{}", String::from_utf8_lossy(&resp.body));
    let body = json(&resp);
    let plugins = body["amxx"]["plugins"].as_array().expect("array");
    assert_eq!(plugins.len(), 4);

    assert_eq!(plugins[0]["file"], "admin.amxx");
    assert_eq!(plugins[0]["group_index"], 0);
    assert_eq!(plugins[0]["group_title"], "Basic");
    assert_eq!(plugins[0]["debug"], false);
    assert_eq!(plugins[0]["comment"], "admin base");

    assert_eq!(plugins[1]["file"], "statsx.amxx");
    assert_eq!(plugins[1]["group_index"], 0);
    assert_eq!(plugins[1]["debug"], true);
    assert_eq!(plugins[1]["comment"], "stats");

    assert_eq!(plugins[2]["file"], "parachute.amxx");
    assert_eq!(plugins[2]["group_index"], 1);
    assert_eq!(plugins[2]["group_title"], "Fun");
    assert_eq!(plugins[2]["debug"], true);
    assert_eq!(plugins[2]["enabled"], false);

    assert_eq!(plugins[3]["file"], "lone.amxx");
    assert!(plugins[3]["group_title"].is_null());
    assert_eq!(plugins[3]["group_index"], 4294967295u64);
}

#[test]
fn attributes_set_debug_and_comment() {
    let mut host = full_setup();
    let resp = dispatch(
        &mut host,
        &request(
            "POST",
            "/servers/3/amxx/plugins/attributes",
            br#"{"file":"statsx.amxx","debug":true,"comment":"my note"}"#,
        ),
    );
    assert_eq!(resp.status_code, 200, "{}", String::from_utf8_lossy(&resp.body));
    let body = json(&resp);
    assert_eq!(body["changed"], true);
    assert_eq!(body["debug"], true);
    assert_eq!(body["comment"], "my note");
    let ini = host
        .file(&format!("{MOD}/addons/amxmodx/configs/plugins.ini"))
        .expect("ini exists");
    assert_eq!(
        ini,
        b"; AMX Mod X plugins\nadmin.amxx\nstatsx.amxx debug ; my note\n;parachute.amxx\n"
    );

    // Turning debug off keeps the comment.
    let resp = dispatch(
        &mut host,
        &request(
            "POST",
            "/servers/3/amxx/plugins/attributes",
            br#"{"file":"statsx.amxx","debug":false,"comment":"my note"}"#,
        ),
    );
    assert_eq!(json(&resp)["changed"], true);
    let ini = host
        .file(&format!("{MOD}/addons/amxmodx/configs/plugins.ini"))
        .expect("ini exists");
    assert_eq!(
        ini,
        b"; AMX Mod X plugins\nadmin.amxx\nstatsx.amxx ; my note\n;parachute.amxx\n"
    );
}

#[test]
fn attributes_unchanged_is_noop() {
    let mut host = full_setup();
    let resp = dispatch(
        &mut host,
        &request(
            "POST",
            "/servers/3/amxx/plugins/attributes",
            br#"{"file":"statsx.amxx","debug":false}"#,
        ),
    );
    assert_eq!(resp.status_code, 200);
    assert_eq!(json(&resp)["changed"], false);
    let ini = host
        .file(&format!("{MOD}/addons/amxmodx/configs/plugins.ini"))
        .expect("ini exists");
    assert_eq!(ini, b"; AMX Mod X plugins\nadmin.amxx\nstatsx.amxx\n;parachute.amxx\n");
}

#[test]
fn attributes_rejects_control_chars_in_comment() {
    let mut host = full_setup();
    let resp = dispatch(
        &mut host,
        &request(
            "POST",
            "/servers/3/amxx/plugins/attributes",
            br#"{"file":"statsx.amxx","comment":"a\nb"}"#,
        ),
    );
    assert_eq!(resp.status_code, 400);
    // The file is untouched.
    let ini = host
        .file(&format!("{MOD}/addons/amxmodx/configs/plugins.ini"))
        .expect("ini exists");
    assert_eq!(ini, b"; AMX Mod X plugins\nadmin.amxx\nstatsx.amxx\n;parachute.amxx\n");
}

#[test]
fn attributes_metamod_sets_description() {
    let mut host = full_setup();
    let resp = dispatch(
        &mut host,
        &request(
            "POST",
            "/servers/3/metamod/plugins/attributes",
            br#"{"file":"reunion_mm_i386.so","comment":"Reunion"}"#,
        ),
    );
    assert_eq!(resp.status_code, 200, "{}", String::from_utf8_lossy(&resp.body));
    assert_eq!(json(&resp)["changed"], true);
    let ini = host
        .file(&format!("{MOD}/addons/metamod/plugins.ini"))
        .expect("ini exists");
    assert!(
        String::from_utf8_lossy(ini)
            .contains("linux addons/reunion/reunion_mm_i386.so Reunion\n"),
        "{}",
        String::from_utf8_lossy(ini)
    );
}

#[test]
fn attributes_protects_amxx_loader_entry() {
    let mut host = full_setup();
    let resp = dispatch(
        &mut host,
        &request(
            "POST",
            "/servers/3/metamod/plugins/attributes",
            br#"{"file":"amxmodx_mm_i386.so","comment":"x"}"#,
        ),
    );
    assert_eq!(resp.status_code, 422);
    assert_eq!(json(&resp)["code"], "SYSTEM_ENTRY");
}

#[test]
fn attributes_unknown_plugin() {
    let mut host = full_setup();
    let resp = dispatch(
        &mut host,
        &request(
            "POST",
            "/servers/3/amxx/plugins/attributes",
            br#"{"file":"nope.amxx","debug":true}"#,
        ),
    );
    assert_eq!(resp.status_code, 404);
    assert_eq!(json(&resp)["code"], "PLUGIN_NOT_REGISTERED");
}
