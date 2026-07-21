<template>
    <div class="mt-2">
        <!-- Fallback guard: the tab itself is game-gated via checkGame -->
        <n-card v-if="!isGoldSource" size="small">
            <div class="py-10">
                <n-empty :description="trans('not_goldsource')" size="small" />
            </div>
        </n-card>

        <template v-else>
            <Loading v-if="loading && !state" />

            <n-card v-else-if="loadError" size="small">
                <div class="py-8 text-center">
                    <div class="text-sm text-red-500 dark:text-red-400 mb-3">{{ loadError }}</div>
                    <GButton color="white" size="small" @click="refreshAll">
                        <GIcon name="refresh" /><span class="ml-1">{{ trans('retry') }}</span>
                    </GButton>
                </div>
            </n-card>

            <template v-else-if="state">
                <!-- rcon hint -->
                <div
                    v-if="rconHint"
                    class="mb-3 flex items-center gap-2 text-xs text-stone-400 dark:text-stone-500"
                >
                    <GIcon name="info" size="sm" />
                    <span>{{ rconHint }}</span>
                </div>

                <!-- platform cards double as tab triggers -->
                <div class="platform-cards grid md:grid-cols-2 gap-3 mb-3" role="tablist">
                    <PlatformCard
                        kind="metamod"
                        :state="state"
                        :version="metaVersion"
                        :rows="metamodRows"
                        :active="activeList === 'metamod'"
                        @select="activeList = 'metamod'"
                    />
                    <PlatformCard
                        kind="amxx"
                        :state="state"
                        :version="amxxVersion"
                        :rows="amxxRows"
                        :active="activeList === 'amxx'"
                        @select="activeList = 'amxx'"
                    />
                </div>

                <!-- plugin lists -->
                <n-card size="small" class="plugins-panel" role="tabpanel">
                    <template v-if="nothingInstalled">
                        <div class="py-12 text-center">
                            <i class="fa-solid fa-puzzle-piece fa-2x text-stone-300 dark:text-stone-600"></i>
                            <div class="mt-3 font-medium text-stone-700 dark:text-stone-200">
                                {{ trans('nothing_installed_title') }}
                            </div>
                            <div class="mt-1 text-sm text-stone-500 dark:text-stone-400 max-w-md mx-auto">
                                {{ trans('nothing_installed_text') }}
                            </div>
                        </div>
                    </template>

                    <template v-else>
                        <PluginList
                            :key="activeList"
                            :kind="activeList"
                            :rows="activeList === 'amxx' ? amxxRows : metamodRows"
                            :installed="activeList === 'amxx' ? state.amxx.installed : metamodPresent"
                            :ini-path="activeList === 'amxx' ? state.paths.amxx_plugins_ini : state.paths.metamod_plugins_ini"
                            :busy="mutating"
                            @toggle="onToggle"
                            @pause="onPause"
                            @set-debug="onSetDebug"
                            @set-comment="onSetComment"
                            @remove="onDelete"
                            @bulk="onBulk"
                            @install="openInstall"
                            @configure="openConfig"
                            @edit-source="openSource"
                            @open-files="openFileManager"
                        />
                    </template>
                </n-card>

                <InstallModal
                    v-model:show="installOpen"
                    :kind="installPlatform"
                    :server-id="serverId"
                    :plugin-id="pluginId"
                    :paths="state.paths"
                    :existing-files="installExistingFiles"
                    @installed="onInstalled"
                />
                <ConfigModal
                    v-model:show="configOpen"
                    :server-id="serverId"
                    :row="configRow"
                />
                <SourceModal
                    v-model:show="sourceOpen"
                    :server-id="serverId"
                    :plugin-id="pluginId"
                    :row="sourceRow"
                    @compiled="onCompiled"
                />
            </template>
        </template>
    </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { NCard, NEmpty } from 'naive-ui';
import { providePluginTrans } from '@gameap/plugin-sdk';

import ConfigModal from './ConfigModal.vue';
import InstallModal from './InstallModal.vue';
import PlatformCard from './PlatformCard.vue';
import PluginList from './PluginList.vue';
import SourceModal from './SourceModal.vue';
import { RconError, amxxSetPaused, rcon } from '../api/gameap';
import { apiErrorMessage, deletePlugin, getState, setAttributes, togglePlugin } from '../api/plugin';
import {
    matchRuntimeToFiles,
    parseAmxxPlugins,
    parseAmxxVersion,
    parseMetaList,
    parseMetaVersion,
} from '../lib/rcon-parse';
import { prettyName, fileStem } from '../lib/naming';
import { computeRowStatus } from '../lib/status';
import type {
    PlatformKind,
    PlatformVersion,
    PluginRow,
    RuntimePluginInfo,
    ServerTabProps,
    StateResponse,
} from '../types';

const props = defineProps<ServerTabProps>();
const { trans } = providePluginTrans(props.pluginId);

const state = ref<StateResponse | null>(null);
const loading = ref(false);
const loadError = ref<string | null>(null);
const mutating = ref(false);

type RconAvailability =
    | 'unknown'
    | 'ok'
    | 'offline'
    | 'no-rcon'
    | 'bad-password'
    | 'empty'
    | 'error';
const rconAvailability = ref<RconAvailability>('unknown');
const metaVersion = ref<PlatformVersion | null>(null);
const amxxVersion = ref<PlatformVersion | null>(null);
const metaRuntime = ref<RuntimePluginInfo[]>([]);
const amxxRuntime = ref<RuntimePluginInfo[]>([]);

const activeList = ref<PlatformKind>('amxx');

const installOpen = ref(false);
const installPlatform = ref<PlatformKind>('amxx');
const installExistingFiles = computed(() =>
    (installPlatform.value === 'amxx' ? amxxRows.value : metamodRows.value).map((row) => row.file),
);
const configOpen = ref(false);
const configRow = ref<PluginRow | null>(null);
const sourceOpen = ref(false);
const sourceRow = ref<PluginRow | null>(null);

const serverGame = computed(() => {
    return (props.server as unknown as { game?: { engine?: string } } | undefined)?.game;
});
const isGoldSource = computed(() => {
    const engine = serverGame.value?.engine;
    // While the server object is still loading, trust the tab-level checkGame gate.
    if (!engine) {
        return true;
    }
    return engine.toLowerCase() === 'goldsource';
});

const serverOnline = computed(() => Boolean(props.server?.process_active));

const metamodPresent = computed(() => {
    if (!state.value) {
        return false;
    }
    return state.value.metamod.installed || state.value.metamod.dir_present;
});

const nothingInstalled = computed(() => {
    if (!state.value) {
        return false;
    }
    return !metamodPresent.value && !state.value.amxx.installed;
});

const rconHint = computed(() => {
    switch (rconAvailability.value) {
        case 'offline':
            return trans('rcon_unavailable_offline');
        case 'no-rcon':
            return trans('rcon_unavailable_norcon');
        case 'bad-password':
            return trans('rcon_unavailable_badpass');
        case 'empty':
            return trans('rcon_unavailable_empty');
        case 'error':
            return trans('rcon_unavailable_error');
        default:
            return null;
    }
});

const rconOk = computed(() => rconAvailability.value === 'ok');

function buildRow(
    kind: PlatformKind,
    file: string,
    iniPath: string,
    enabled: boolean,
    missing: boolean,
    system: boolean,
    hasConfig: boolean,
    configPath: string | null,
    fallbackName: string,
    runtime: RuntimePluginInfo | null,
): Omit<PluginRow, 'debug' | 'comment' | 'groupIndex' | 'groupTitle' | 'hasSource' | 'sourcePath'> {
    const { status, detail } = computeRowStatus({
        enabled,
        missing,
        runtime,
        rconOk: rconOk.value,
    });

    return {
        key: `${kind}:${file}`,
        file,
        iniPath,
        name: runtime?.name ?? fallbackName,
        version: runtime?.version ?? null,
        author: runtime?.author ?? null,
        enabled,
        missing,
        system,
        runtime,
        hasConfig,
        configPath,
        status,
        statusDetail: detail,
    };
}

const amxxRows = computed<PluginRow[]>(() => {
    if (!state.value) {
        return [];
    }
    const entries = state.value.amxx.plugins;
    const runtimes = matchRuntimeToFiles(
        entries.map((entry) => entry.file),
        amxxRuntime.value,
    );
    return entries.map((entry, index) => ({
        ...buildRow(
            'amxx',
            entry.file,
            entry.file,
            entry.enabled,
            entry.missing,
            false,
            entry.has_config,
            entry.config_path,
            prettyName(entry.file),
            runtimes[index],
        ),
        debug: entry.debug,
        comment: entry.comment,
        hasSource: entry.has_source,
        sourcePath: entry.has_source
            ? `${state.value.paths.amxx_scripting_dir}/${fileStem(entry.file)}.sma`
            : null,
        groupIndex: entry.group_index,
        groupTitle: entry.group_title,
    }));
});

const metamodRows = computed<PluginRow[]>(() => {
    if (!state.value) {
        return [];
    }
    const entries = state.value.metamod.plugins;
    const runtimes = matchRuntimeToFiles(
        entries.map((entry) => entry.file),
        metaRuntime.value,
    );
    return entries.map((entry, index) => ({
        ...buildRow(
            'metamod',
            entry.file,
            entry.path,
            entry.enabled,
            entry.missing,
            entry.system,
            false,
            null,
            entry.description ?? prettyName(entry.file),
            runtimes[index],
        ),
        debug: false,
        comment: entry.description,
        hasSource: false,
        sourcePath: null,
        groupIndex: entry.group_index,
        groupTitle: entry.group_title,
    }));
});

async function refreshState(): Promise<void> {
    loading.value = true;
    loadError.value = null;
    try {
        state.value = await getState(props.pluginId, props.serverId);
    } catch (error) {
        loadError.value = apiErrorMessage(error, trans('load_failed'));
    } finally {
        loading.value = false;
    }
}

async function refreshRcon(): Promise<void> {
    if (!serverOnline.value) {
        rconAvailability.value = 'offline';
        metaVersion.value = null;
        amxxVersion.value = null;
        metaRuntime.value = [];
        amxxRuntime.value = [];
        return;
    }
    try {
        const [metaVersionOut, amxxVersionOut, metaListOut, amxxPluginsOut] = [
            await rcon(props.serverId, 'meta version'),
            await rcon(props.serverId, 'amxx version'),
            await rcon(props.serverId, 'meta list'),
            await rcon(props.serverId, 'amxx plugins'),
        ];
        metaVersion.value = parseMetaVersion(metaVersionOut);
        amxxVersion.value = parseAmxxVersion(amxxVersionOut);
        metaRuntime.value = parseMetaList(metaListOut);
        amxxRuntime.value = parseAmxxPlugins(amxxPluginsOut);
        rconAvailability.value = 'ok';
    } catch (error) {
        applyRconFailure(error);
    }
}

/** Mark the console unavailable after a failed RCON call and drop runtime data. */
function applyRconFailure(error: unknown): void {
    rconAvailability.value = error instanceof RconError ? error.reason : 'error';
    metaVersion.value = null;
    amxxVersion.value = null;
    metaRuntime.value = [];
    amxxRuntime.value = [];
}

async function refreshAll(): Promise<void> {
    await Promise.all([refreshState(), refreshRcon()]);
}

function toast(type: 'success' | 'error' | 'info', text: string): void {
    window.$message?.[type]?.(text);
}

async function onToggle(kind: PlatformKind, row: PluginRow, value: boolean): Promise<void> {
    mutating.value = true;
    try {
        await togglePlugin(props.pluginId, props.serverId, kind, row.file, value);
        toast('success', trans(value ? 'toggled_on' : 'toggled_off', { name: row.name }));
        await refreshState();
    } catch (error) {
        toast('error', apiErrorMessage(error, trans('op_failed')));
    } finally {
        mutating.value = false;
    }
}

async function onPause(kind: PlatformKind, row: PluginRow, paused: boolean): Promise<void> {
    mutating.value = true;
    try {
        const output = await amxxSetPaused(props.serverId, row.file, paused);
        amxxRuntime.value = parseAmxxPlugins(await rcon(props.serverId, 'amxx plugins'));
        rconAvailability.value = 'ok';
        // Rows are matched in load order — a sibling with the same truncated
        // file name must not shadow this plugin's own runtime entry.
        const runtime = amxxRows.value.find((item) => item.file === row.file)?.runtime ?? null;
        const expected = paused ? 'paused' : 'running';
        if (runtime?.status === expected) {
            toast('success', trans(paused ? 'paused_ok' : 'unpaused_ok', { name: row.name }));
        } else {
            toast(
                'error',
                output || trans(paused ? 'pause_failed' : 'unpause_failed', { name: row.name }),
            );
        }
    } catch (error) {
        applyRconFailure(error);
        toast(
            'error',
            error instanceof RconError && error.reason === 'bad-password'
                ? trans('rcon_unavailable_badpass')
                : apiErrorMessage(error, trans('op_failed')),
        );
    } finally {
        mutating.value = false;
    }
}

async function onSetDebug(kind: PlatformKind, row: PluginRow, value: boolean): Promise<void> {
    mutating.value = true;
    try {
        await setAttributes(props.pluginId, props.serverId, kind, row.file, value, row.comment);
        toast('success', trans(value ? 'debug_on' : 'debug_off', { name: row.name }));
        await refreshState();
    } catch (error) {
        toast('error', apiErrorMessage(error, trans('op_failed')));
    } finally {
        mutating.value = false;
    }
}

async function onSetComment(kind: PlatformKind, row: PluginRow, text: string): Promise<void> {
    const comment = text.trim() || null;
    // A comment is cosmetic — skip a no-op write.
    if (comment === (row.comment ?? null)) {
        return;
    }
    mutating.value = true;
    try {
        await setAttributes(props.pluginId, props.serverId, kind, row.file, row.debug, comment);
        toast('success', trans('comment_saved', { name: row.name }));
        await refreshState();
    } catch (error) {
        toast('error', apiErrorMessage(error, trans('op_failed')));
    } finally {
        mutating.value = false;
    }
}

function onDelete(kind: PlatformKind, row: PluginRow): void {
    window.$dialog?.success({
        title: trans('delete_title', { name: row.name }),
        content: trans(kind === 'amxx' ? 'delete_text_amxx' : 'delete_text_metamod'),
        positiveText: trans('yes'),
        negativeText: trans('no'),
        closable: false,
        onPositiveClick: async () => {
            mutating.value = true;
            try {
                await deletePlugin(props.pluginId, props.serverId, kind, row.file);
                toast('success', trans('deleted', { name: row.name }));
                await refreshState();
            } catch (error) {
                toast('error', apiErrorMessage(error, trans('op_failed')));
            } finally {
                mutating.value = false;
            }
        },
    });
}

async function applyBulkToggle(kind: PlatformKind, rows: PluginRow[], value: boolean): Promise<void> {
    mutating.value = true;
    let changed = 0;
    try {
        for (const row of rows) {
            if (row.system || row.enabled === value) {
                continue;
            }
            await togglePlugin(props.pluginId, props.serverId, kind, row.file, value);
            changed += 1;
        }
        if (changed > 0) {
            toast('success', trans(value ? 'bulk_enabled' : 'bulk_disabled', { count: changed }));
            await refreshState();
        }
    } catch (error) {
        toast('error', apiErrorMessage(error, trans('op_failed')));
        await refreshState();
    } finally {
        mutating.value = false;
    }
}

function onBulk(kind: PlatformKind, action: 'enable' | 'disable' | 'delete', rows: PluginRow[]): void {
    if (action !== 'delete') {
        void applyBulkToggle(kind, rows, action === 'enable');
        return;
    }
    const deletable = rows.filter((row) => !row.system);
    if (deletable.length === 0) {
        return;
    }
    window.$dialog?.success({
        title: trans('bulk_delete_title', { count: deletable.length }),
        content: trans('bulk_delete_text'),
        positiveText: trans('yes'),
        negativeText: trans('no'),
        closable: false,
        onPositiveClick: async () => {
            mutating.value = true;
            let deleted = 0;
            try {
                for (const row of deletable) {
                    await deletePlugin(props.pluginId, props.serverId, kind, row.file);
                    deleted += 1;
                }
                toast('success', trans('bulk_deleted', { count: deleted }));
            } catch (error) {
                toast('error', apiErrorMessage(error, trans('op_failed')));
            } finally {
                await refreshState();
                mutating.value = false;
            }
        },
    });
}

function openInstall(kind: PlatformKind): void {
    installPlatform.value = kind;
    installOpen.value = true;
}

async function onInstalled(): Promise<void> {
    await refreshState();
}

function openConfig(row: PluginRow): void {
    configRow.value = row;
    configOpen.value = true;
}

function openSource(row: PluginRow): void {
    sourceRow.value = row;
    sourceOpen.value = true;
}

async function onCompiled(): Promise<void> {
    await refreshState();
}

function openFileManager(): void {
    window.location.hash = '#files';
}

// The server object can arrive after the tab mounts (async store load) —
// re-query the console once the server turns out to be online.
watch(serverOnline, (online, wasOnline) => {
    if (online && !wasOnline) {
        void refreshRcon();
    }
});

onMounted(() => {
    if (isGoldSource.value) {
        void refreshAll();
    }
});
</script>

<style scoped>
/* On md+ screens the platform cards sit flush above this panel and the active
 * one reads as a tab merged into it: no gap between the row and the panel, no
 * top border or rounding on the panel (the active card drops its bottom
 * border, the inactive one keeps it as the tab-row line). Doubled selectors
 * beat naive-ui's runtime-injected .n-card styles; the margin lives here
 * because the host panel's prebuilt Tailwind may lack arbitrary utilities. */
@media (min-width: 768px) {
    .platform-cards {
        margin-bottom: 0;
    }

    .plugins-panel.plugins-panel {
        border-top: none;
        border-top-left-radius: 0;
        border-top-right-radius: 0;
    }
}
</style>
