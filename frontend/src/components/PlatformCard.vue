<template>
    <n-card size="small">
        <div class="flex items-start gap-3">
            <div
                class="w-10 h-10 rounded-lg bg-stone-700 dark:bg-stone-900 text-white flex items-center justify-center flex-shrink-0"
            >
                <i :class="isMetamod ? 'fa-solid fa-plug fa-lg' : 'fa-solid fa-puzzle-piece fa-lg'"></i>
            </div>
            <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2 flex-wrap">
                    <span class="font-semibold text-stone-800 dark:text-stone-100">{{ title }}</span>
                    <n-tooltip v-if="notActive" trigger="hover">
                        <template #trigger>
                            <GStatusBadge status="warning" :text="trans('status_not_active')" />
                        </template>
                        {{ trans('not_active_hint') }}
                    </n-tooltip>
                    <GStatusBadge
                        v-else-if="!installed"
                        status="error"
                        :text="trans('status_not_installed')"
                    />
                </div>
                <div
                    v-if="installed || notActive"
                    class="text-xs text-stone-500 dark:text-stone-400 font-mono mt-0.5 truncate"
                >
                    <template v-if="version">v{{ version.version }} · </template>
                    <template v-else>{{ trans('version_unknown') }} · </template>{{ dirPath }}
                </div>
                <div v-else class="text-xs text-stone-500 dark:text-stone-400 mt-0.5">
                    {{ trans(isMetamod ? 'metamod_desc' : 'amxx_desc') }}
                </div>
            </div>
        </div>

        <template v-if="installed || notActive">
            <div class="mt-3 flex flex-wrap gap-x-6 gap-y-1 text-sm text-stone-600 dark:text-stone-300">
                <span>
                    {{ trans('stats_total') }}:
                    <span class="font-medium text-stone-800 dark:text-stone-100">{{ rows.length }}</span>
                </span>
                <span>
                    {{ trans('stats_enabled') }}:
                    <span class="font-medium text-stone-800 dark:text-stone-100">{{ enabledCount }}</span>
                </span>
                <span v-if="errorCount" class="text-red-500 dark:text-red-400">
                    {{ trans('stats_errors') }}: {{ errorCount }}
                </span>
            </div>
        </template>

        <template v-else>
            <div class="mt-3 text-sm text-stone-500 dark:text-stone-400">
                {{ trans(isMetamod ? 'install_hint_metamod' : 'install_hint_amxx') }}
            </div>
        </template>
    </n-card>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { NCard, NTooltip } from 'naive-ui';
import { usePluginTrans } from '@gameap/plugin-sdk';

import type { PlatformKind, PlatformVersion, PluginRow, StateResponse } from '../types';

const props = defineProps<{
    kind: PlatformKind;
    state: StateResponse;
    version: PlatformVersion | null;
    rows: PluginRow[];
}>();

const { trans } = usePluginTrans();

const isMetamod = computed(() => props.kind === 'metamod');

const installed = computed(() =>
    isMetamod.value ? props.state.metamod.installed : props.state.amxx.installed,
);

/** Metamod addons dir exists, but liblist.gam does not load it. */
const notActive = computed(
    () => isMetamod.value && !props.state.metamod.installed && props.state.metamod.dir_present,
);

const title = computed(() => {
    if (props.version) {
        return props.version.build;
    }
    return isMetamod.value ? 'Metamod' : 'AMX Mod X';
});

const dirPath = computed(() =>
    isMetamod.value ? props.state.paths.metamod_dir : props.state.paths.amxx_dir,
);

const enabledCount = computed(() => props.rows.filter((row) => row.enabled).length);
const errorCount = computed(
    () => props.rows.filter((row) => row.status === 'error' || row.status === 'missing').length,
);
</script>
