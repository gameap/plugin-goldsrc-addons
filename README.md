# GameAP GoldSource Addons

GameAP plugin for managing Metamod and AMX Mod X on GoldSource servers
(Half-Life 1, Counter-Strike 1.6, etc.). It adds a "Plugins" tab to the server
page: platform status, the plugin list from `plugins.ini` grouped by section,
enable/disable, per-plugin `debug` flag and comments, delete, install from file
and config editing.

- **Backend** — a Rust WASM module ([gameap-plugin-sdk](https://github.com/gameap/gameap-proto)):
  it assembles the server state and atomically edits `plugins.ini` / reads
  `liblist.gam` through the host `nodefs` library.
- **Frontend** — Vue 3 + naive-ui, embedded into the WASM module. It uses the
  existing panel endpoints: RCON (`meta version`, `amxx version`, `meta list`,
  `amxx plugins` — versions and runtime statuses), the file manager (uploading
  plugin files, configs) and server restart.

## Features

- Metamod / AMX Mod X cards: installed · inactive · not installed, version from
  the server console, plugin counters.
- Plugin table (AMXX and Metamod): status (Running / Awaiting restart /
  Error `bad load` / File missing / Disabled), version and author from the
  console, search, filter, bulk actions.
- Grouping by `plugins.ini` sections (`; Basic`, `; Menus`, …): each named
  section becomes a group; plugins that are not under any header (single plugins
  and header-less blocks) are collected into a common "Other" group. When no
  section is recognizable the list falls back to a flat view.
- Per-plugin comments (the inline `; ...` after an entry) are shown and can be
  edited; the `debug` load flag of AMX Mod X plugins is shown and can be toggled.
- Enable/disable — commenting the line in `plugins.ini`, byte-preserving the
  rest of the file (CRLF, BOM, CP1251 comments).
- Pause/unpause of an AMXX plugin from the list — the `amxx pause <file>` /
  `amxx unpause <file>` console commands via RCON, applied instantly, without
  a restart; the button is available when the server is online and RCON is
  working.
- Install from file: `.amxx` for AMX Mod X, `.so`/`.dll` for Metamod (the file
  is uploaded through the file manager and then registered in the ini). `.sma`
  is rejected with a hint to compile it first. Re-uploading an already installed
  plugin overwrites the file with the new version (`force`); the `plugins.ini`
  line stays in place.
- Editing a plugin config (`configs/<name>.cfg|.ini`) in a modal.
- "Restart required" banner with a server restart button.

Installing / updating / removing Metamod and AMX Mod X themselves is out of the
plugin's scope.

## Tab visibility

The tab is shown only:

- on servers with the **GoldSource** engine (`game.engine == "GoldSource"`) —
  via the `checkGame` of the `server-tabs` slot (requires gameap-api with
  `checkGame` support, plugin-sdk ≥ 0.3.3);
- to users with the `plugin:ezvdsxmlu6fbk:manage` ability — admins get it
  automatically. Do not grant it to regular users: the plugin's HTTP routes are
  declared `admin_only`, so a non-admin will see the tab but get a 403.

## Build

Requires Rust (the toolchain and target are pinned in `rust-toolchain.toml`:
1.94.0 + `wasm32-wasip1`), Node.js and a local copy of
[gameap-proto](https://github.com/gameap/gameap-proto) next to this repository
(the path dependency `../gameap-proto/rust/gameap-plugin-sdk`), as well as
gameap-api next to it (`../gameap-api` — the npm dependency
`@gameap/plugin-sdk`).

```sh
make build     # frontend (npm ci && vite build) + wasm; artifact goldsrc-addons.wasm
make test      # cargo test (ini/liblist/handlers) + vitest (RCON parsers)
make lint      # clippy for native and wasm32-wasip1
```

Order matters: the frontend is built before the wasm — `build.rs` embeds
`frontend/dist/plugin.js` into the module (without `dist` a UI-less module is
built).

## Installation

Copy `goldsrc-addons.wasm` into the panel's plugins directory (or upload it via
Administration → Plugins) and restart the panel.

Plugin API: `/api/plugins/ezvdsxmlu6fbk/...`

| Method | Path | Purpose |
|---|---|---|
| GET | `/servers/{id}/state` | Metamod/AMXX state and plugin lists |
| POST | `/servers/{id}/{platform}/plugins/toggle` | `{file, enabled}` — enable/disable an entry |
| POST | `/servers/{id}/{platform}/plugins/attributes` | `{file, debug, comment}` — set the `debug` flag and inline comment (AMXX; for Metamod only the comment/description is applied, `debug` is ignored) |
| POST | `/servers/{id}/{platform}/plugins` | `{file, enable, path?, force?}` — register an uploaded file; `force: true` overwrites an existing entry (200 instead of 409) |
| DELETE | `/servers/{id}/{platform}/plugins` | `{file}` or `?file=` — remove the entry and the file |

`{platform}` is `amxx` or `metamod`; all routes require an administrator.

## Notes

- The plugin id is `ezvdsxmlu6fbk` (no hyphen): the panel normalizes ids via
  `CompactPluginID`, and hyphenated ids are rewritten to a hash, breaking the
  API paths.
- `plugins.ini` edits are read-modify-write without locking (last-write-wins);
  editing the file concurrently by other means may be overwritten. Enable/disable
  preserves the rest of the file byte-for-byte; changing an entry's `debug` flag
  or comment rewrites only that single line (its whitespace is normalized).
- Console versions and statuses are available only when the server is running
  and an RCON password is set; without them, statuses are shown based on
  `plugins.ini`.
- Console diagnostics: a wrong RCON password (a "Bad Password" reply from
  GoldSource / HTTP 422 from the panel) and an unexpectedly empty console
  response are reported as a separate hint instead of silent empty statuses.
