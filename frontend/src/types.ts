// Mirrors of the Rust backend DTOs (src/model.rs).

export type PlatformKind = 'amxx' | 'metamod';

export interface StatePaths {
    liblist: string;
    metamod_dir: string;
    metamod_plugins_ini: string;
    amxx_dir: string;
    amxx_plugins_ini: string;
    amxx_plugins_dir: string;
    amxx_configs_dir: string;
    amxx_scripting_dir: string;
}

export interface MetamodPluginEntry {
    platform: string;
    path: string;
    file: string;
    description: string | null;
    enabled: boolean;
    missing: boolean;
    system: boolean;
    group_index: number;
    group_title: string | null;
}

export interface AmxxPluginEntry {
    file: string;
    debug: boolean;
    comment: string | null;
    enabled: boolean;
    missing: boolean;
    has_config: boolean;
    config_path: string | null;
    has_source: boolean;
    group_index: number;
    group_title: string | null;
}

export interface MetamodState {
    installed: boolean;
    dir_present: boolean;
    plugins_ini_exists: boolean;
    plugins: MetamodPluginEntry[];
}

export interface AmxxState {
    installed: boolean;
    registered_in_metamod: boolean;
    plugins_ini_exists: boolean;
    plugins: AmxxPluginEntry[];
}

export interface StateResponse {
    server_id: number;
    game_code: string;
    engine: string;
    mod_dir: string;
    paths: StatePaths;
    metamod: MetamodState;
    amxx: AmxxState;
}

// Runtime info assembled from RCON output.

export interface PlatformVersion {
    build: string;
    version: string;
}

export interface RuntimePluginInfo {
    /** File name as printed (possibly truncated by column width). */
    file: string;
    name: string;
    version: string | null;
    author: string | null;
    /** Normalized runtime state. */
    status: 'running' | 'paused' | 'error';
    rawStatus: string;
}

/** Row model the plugin table renders. */
export interface PluginRow {
    key: string;
    file: string;
    /** Ini path for metamod entries (as written), file name for AMXX. */
    iniPath: string;
    name: string;
    version: string | null;
    author: string | null;
    enabled: boolean;
    /** AMX Mod X `debug` load flag (always false for Metamod). */
    debug: boolean;
    /** Inline comment / metamod description; editable. */
    comment: string | null;
    missing: boolean;
    system: boolean;
    runtime: RuntimePluginInfo | null;
    hasConfig: boolean;
    configPath: string | null;
    /** A matching .sma exists in the amxmodx scripting dir (AMXX only). */
    hasSource: boolean;
    /** Panel file-manager path of the .sma source, when hasSource. */
    sourcePath: string | null;
    status: RowStatus;
    statusDetail: string | null;
    /** Display group id; unnamed entries share one trailing "Other" group. */
    groupIndex: number;
    /** Display group header, `null` for the common "Other" group. */
    groupTitle: string | null;
}

export type RowStatus =
    | 'running'
    | 'enabled'
    | 'stopped'
    | 'pending'
    | 'error'
    | 'missing';

// Mirrors of the compile endpoint DTOs.

export interface CompileDiagnostic {
    severity: string;
    code: number;
    line: number;
    line_end: number | null;
    message: string;
}

export interface CompileResponse {
    file: string;
    success: boolean;
    exit_code: number;
    output: string;
    diagnostics: CompileDiagnostic[];
    amxx_file: string | null;
}

// Local mirror of the SDK's ServerData / ServerTabProps contract.
//
// Declared here rather than imported from @gameap/plugin-sdk so that
// `defineProps<ServerTabProps>()` compiles against the shipped plugin build: the
// Vue SFC compiler statically resolves the props type at build time, and the CI
// SDK build runs vite only (its `tsc --emitDeclarationOnly` step fails on
// @gameap/ui and emits no declarations), so the SDK's types are not on disk to
// resolve against.

export interface ServerData {
    id: number;
    uuid: string;
    name: string;
    game_id: string;
    game_mod_id: number;
    ip: string;
    port: number;
    query_port: number;
    rcon_port: number;
    enabled: boolean;
    installed: boolean;
    blocked: boolean;
    start_command: string;
    dir: string;
    process_active: boolean;
    last_process_check: string;
}

export interface ServerTabProps {
    serverId: number;
    server: ServerData;
    pluginId: string;
}
