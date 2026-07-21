<template>
    <n-card
        size="small"
        role="tab"
        tabindex="0"
        :aria-selected="active"
        class="platform-card"
        :class="active ? 'platform-card--active' : 'platform-card--inactive'"
        @click="emit('select')"
        @keydown.enter.prevent="emit('select')"
        @keydown.space.prevent="emit('select')"
    >
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
    active?: boolean;
}>();

const emit = defineEmits<{
    (e: 'select'): void;
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

<style scoped>
/*
 * The card doubles as a tab trigger. On md+ screens the active card merges with
 * the plugin-list panel below it: no bottom rounding, no bottom border, and the
 * panel has no top border (see ModsTab.vue) — so nothing separates the two.
 * The inactive card keeps its bottom border, which reads as the tab-row line.
 * Doubled class selectors keep specificity above naive-ui's runtime-injected
 * .n-card styles.
 */
.platform-card {
    cursor: pointer;
    user-select: none;
    transition:
        background-color 0.2s ease,
        box-shadow 0.2s ease;
}

.platform-card:focus-visible {
    outline: 2px solid #22c55e;
    outline-offset: 1px;
}

/* Narrow screens: cards are stacked, so the active one gets an accent ring. */
.platform-card.platform-card--active {
    box-shadow: inset 0 0 0 2px #57534e;
}

.platform-card.platform-card--inactive:hover {
    box-shadow: inset 0 0 0 1px #d6d3d1;
}

@media (min-width: 768px) {
    .platform-card {
        border-bottom-left-radius: 0;
        border-bottom-right-radius: 0;
    }

    .platform-card.platform-card--active {
        border-bottom-color: transparent;
        box-shadow: none;
    }

    .platform-card.platform-card--inactive {
        background-color: #f5f5f4;
    }

    .platform-card.platform-card--inactive:hover {
        background-color: #ffffff;
        box-shadow: none;
    }
}
</style>

<!-- Dark-theme variants. The host panel toggles .dark on <html>, and scoped
     :global() selectors get mangled by the build — keep these unscoped. -->
<style>
.dark .platform-card.platform-card--active {
    box-shadow: inset 0 0 0 2px #d6d3d1;
}

.dark .platform-card.platform-card--inactive:hover {
    box-shadow: inset 0 0 0 1px #57534e;
}

@media (min-width: 768px) {
    .dark .platform-card.platform-card--active {
        box-shadow: none;
    }

    .dark .platform-card.platform-card--inactive {
        background-color: rgba(12, 10, 9, 0.6);
    }

    .dark .platform-card.platform-card--inactive:hover {
        background-color: rgba(12, 10, 9, 0.1);
    }
}
</style>
