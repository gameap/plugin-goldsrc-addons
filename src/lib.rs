//! GameAP plugin: manage Metamod and AMX Mod X plugins on GoldSource servers.
//!
//! Backend of the "Плагины" server tab: assembles Metamod/AMXX state and
//! performs plugins.ini mutations through the nodefs host library. The Vue
//! frontend (embedded bundle) talks to these routes and to existing panel
//! endpoints (RCON for versions, file-manager for uploads and configs).

#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

pub mod goldsrc;
pub mod handlers;
pub mod host_api;
pub mod http;
pub mod model;
pub mod router;

use gameap_plugin_sdk::proto::gameap::plugin as pb;
use gameap_plugin_sdk::{Plugin, PluginError, register_plugin};

use crate::host_api::HostApi;

// The panel normalizes plugin ids (CompactPluginID); "ezvdsxmlu6fbk" is
// round-trip stable, so /api/plugins/ezvdsxmlu6fbk/... works literally.
// A hyphenated id would be rewritten to an FNV hash and break route paths.
pub const PLUGIN_ID: &str = "ezvdsxmlu6fbk";

const FRONTEND_JS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/plugin.js"));
const FRONTEND_CSS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/plugin.css"));

pub struct GoldsrcAddons<H> {
    host: H,
}

impl<H> GoldsrcAddons<H> {
    pub fn new(host: H) -> Self {
        Self { host }
    }
}

impl<H: HostApi> Plugin for GoldsrcAddons<H> {
    fn get_info(&mut self, _req: pb::GetInfoRequest) -> Result<pb::PluginInfo, PluginError> {
        Ok(pb::PluginInfo {
            id: PLUGIN_ID.into(),
            name: "GoldSource Addons".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            description: "Manage Metamod and AMX Mod X plugins on GoldSource servers".into(),
            author: "GameAP".into(),
            license: "MIT".into(),
            api_version: "1".into(),
            ..Default::default()
        })
    }

    fn initialize(
        &mut self,
        _req: pb::InitializeRequest,
    ) -> Result<pb::InitializeResponse, PluginError> {
        self.host.log_info("goldsrc-addons plugin initialized");
        Ok(pb::InitializeResponse {
            result: Some(gameap_plugin_sdk::ok_result()),
        })
    }

    fn get_http_routes(
        &mut self,
        _req: pb::GetHttpRoutesRequest,
    ) -> Result<pb::GetHttpRoutesResponse, PluginError> {
        Ok(pb::GetHttpRoutesResponse {
            routes: router::http_routes(),
        })
    }

    fn handle_http_request(
        &mut self,
        req: pb::HttpRequest,
    ) -> Result<pb::HttpResponse, PluginError> {
        // Total dispatch: every failure becomes a JSON error response. An Err
        // here would surface as a plain-text host 500.
        Ok(router::dispatch(&mut self.host, &req))
    }

    fn get_server_abilities(
        &mut self,
        _req: pb::GetServerAbilitiesRequest,
    ) -> Result<pb::GetServerAbilitiesResponse, PluginError> {
        // Admins get plugin abilities automatically; the frontend tab is
        // gated on plugin:ezvdsxmlu6fbk:manage.
        Ok(pb::GetServerAbilitiesResponse {
            abilities: vec![pb::ServerAbility {
                name: "manage".into(),
                title: "Manage GoldSource addons (Metamod / AMX Mod X)".into(),
            }],
        })
    }

    fn get_frontend_bundle(
        &mut self,
        _req: pb::GetFrontendBundleRequest,
    ) -> Result<pb::GetFrontendBundleResponse, PluginError> {
        Ok(pb::GetFrontendBundleResponse {
            bundle: FRONTEND_JS.to_vec(),
            has_bundle: !FRONTEND_JS.is_empty(),
            styles: FRONTEND_CSS.to_vec(),
            has_styles: !FRONTEND_CSS.is_empty(),
        })
    }
}

register_plugin!(
    GoldsrcAddons<host_api::WasmHost>,
    GoldsrcAddons::new(host_api::WasmHost)
);
