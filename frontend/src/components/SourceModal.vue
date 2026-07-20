<template>
    <GModal
        :show="show"
        :title="title"
        :style="{ width: '760px' }"
        transform-origin="center"
        @update:show="(value: boolean) => $emit('update:show', value)"
    >
        <div
            v-if="row?.sourcePath"
            class="mb-2 text-xs text-stone-400 dark:text-stone-500 font-mono flex items-center gap-1.5"
        >
            <GIcon name="file-lines" size="sm" /> {{ row.sourcePath }}
        </div>

        <Loading v-if="loading" />
        <textarea
            v-else
            ref="editor"
            v-model="text"
            rows="18"
            spellcheck="false"
            class="w-full text-xs font-mono px-2 py-1.5 rounded border border-stone-300 dark:border-stone-600 bg-white dark:bg-stone-800 text-stone-700 dark:text-stone-200 focus:outline-none focus:ring-1 focus:ring-emerald-400"
        ></textarea>

        <!-- compile status: the alert always stays in the DOM (hidden while
             idle) so its exact height is reserved and the editor doesn't jump.
             Inline style: the plugin CSS bundle is not guaranteed to apply. -->
        <n-alert
            class="mt-3"
            :style="{ visibility: statusIdle ? 'hidden' : 'visible' }"
            :type="statusType"
            :show-icon="true"
        >
            <span v-if="compiling" class="inline-flex items-center gap-2">
                <GIcon name="spinner" class="fa-spin" /> {{ trans('compiling') }}
            </span>
            <template v-else-if="result?.success">
                {{ trans('compile_success', { name: result.amxx_file ?? '' }) }}
            </template>
            <template v-else-if="result">{{ trans('compile_failed') }}</template>
            <template v-else>&nbsp;</template>
        </n-alert>

        <div v-if="result && !result.success">
            <ul v-if="result.diagnostics.length" class="mt-2 space-y-1 text-xs font-mono">
                <li v-for="(diag, idx) in result.diagnostics" :key="idx">
                    <button
                        class="text-left w-full px-2 py-1 rounded hover:bg-stone-100 dark:hover:bg-stone-700 text-stone-700 dark:text-stone-200"
                        :title="trans('compile_goto_line')"
                        @click="goToLine(diag.line)"
                    >
                        <span class="text-stone-400">{{ diag.severity === 'warning' ? 'W' : 'E' }}{{ diag.code }}</span>
                        <span class="mx-1 text-orange-500">:{{ diag.line }}</span>
                        {{ diag.message }}
                    </button>
                </li>
            </ul>
            <details v-if="result.output" class="mt-2">
                <summary class="text-xs text-stone-400 cursor-pointer">{{ trans('compile_log') }}</summary>
                <pre class="mt-1 p-2 rounded bg-stone-100 dark:bg-stone-900 text-xs font-mono whitespace-pre-wrap text-stone-600 dark:text-stone-300 max-h-48 overflow-auto">{{ result.output }}</pre>
            </details>
        </div>

        <template #footer>
            <div class="flex gap-2 justify-end">
                <GButton color="white" :disabled="loading || saving || compiling" @click="save">
                    <GIcon name="save" class="mr-1" />
                    {{ trans('save') }}
                </GButton>
                <GButton color="green" :disabled="loading || saving || compiling" @click="compile">
                    <i v-if="!compiling" class="fa-solid fa-hammer mr-1"></i>
                    <GIcon v-else name="spinner" class="mr-1 fa-spin" />
                    {{ compiling ? trans('compiling') : trans('compile') }}
                </GButton>
            </div>
        </template>
    </GModal>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { NAlert } from 'naive-ui';
import { usePluginTrans } from '@gameap/plugin-sdk';

import { fmDownloadText, fmUploadFile } from '../api/gameap';
import { apiErrorMessage, compileSource } from '../api/plugin';
import { lineOffset } from '../lib/source';
import type { CompileResponse, PluginRow } from '../types';

const props = defineProps<{
    show: boolean;
    serverId: number;
    pluginId: string;
    row: PluginRow | null;
}>();

const emit = defineEmits<{
    'update:show': [value: boolean];
    compiled: [];
}>();

const { trans } = usePluginTrans();

const text = ref('');
const loading = ref(false);
const saving = ref(false);
const compiling = ref(false);
const result = ref<CompileResponse | null>(null);
const editor = ref<HTMLTextAreaElement | null>(null);

const statusIdle = computed(() => !compiling.value && result.value === null);
const statusType = computed(() => {
    if (compiling.value || statusIdle.value) {
        return 'info';
    }
    return result.value?.success ? 'success' : 'error';
});

const title = computed(() =>
    props.row ? trans('source_title', { name: props.row.name }) : trans('action_source'),
);

watch(
    () => props.show,
    async (shown) => {
        result.value = null;
        if (!shown || !props.row?.sourcePath) {
            return;
        }
        loading.value = true;
        text.value = '';
        try {
            text.value = await fmDownloadText(props.serverId, props.row.sourcePath);
        } catch (error) {
            window.$message?.error(apiErrorMessage(error, trans('source_load_failed')));
            emit('update:show', false);
        } finally {
            loading.value = false;
        }
    },
);

/** Writes the editor content back to the .sma file. */
async function saveText(): Promise<void> {
    const sourcePath = props.row?.sourcePath;
    if (!sourcePath) {
        return;
    }
    const idx = sourcePath.lastIndexOf('/');
    const directory = idx > 0 ? sourcePath.slice(0, idx) : '.';
    const name = idx > 0 ? sourcePath.slice(idx + 1) : sourcePath;
    await fmUploadFile(props.serverId, directory, new File([text.value], name, { type: 'text/plain' }));
}

async function save(): Promise<void> {
    saving.value = true;
    try {
        await saveText();
        window.$message?.success(trans('source_saved'));
    } catch (error) {
        window.$message?.error(apiErrorMessage(error, trans('op_failed')));
    } finally {
        saving.value = false;
    }
}

/** Compile runs against the file on the node, so save first. */
async function compile(): Promise<void> {
    const sourcePath = props.row?.sourcePath;
    if (!sourcePath) {
        return;
    }
    const name = sourcePath.slice(sourcePath.lastIndexOf('/') + 1);
    compiling.value = true;
    result.value = null;
    try {
        await saveText();
        result.value = await compileSource(props.pluginId, props.serverId, name);
        if (result.value.success) {
            emit('compiled');
        }
    } catch (error) {
        window.$message?.error(apiErrorMessage(error, trans('op_failed')));
    } finally {
        compiling.value = false;
    }
}

function goToLine(line: number): void {
    const el = editor.value;
    if (!el) {
        return;
    }
    const offset = lineOffset(text.value, line);
    el.focus();
    el.setSelectionRange(offset, offset);
    // Bring the line into view: approximate by line height.
    const lineHeight = parseFloat(getComputedStyle(el).lineHeight) || 16;
    el.scrollTop = Math.max(0, (line - 5) * lineHeight);
}
</script>
