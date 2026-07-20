<template>
    <GModal
        :show="show"
        :title="trans(kind === 'amxx' ? 'install_title_amxx' : 'install_title_metamod')"
        :style="{ width: '600px' }"
        transform-origin="center"
        @update:show="onUpdateShow"
    >
        <div class="space-y-3">
            <n-upload
                v-if="!file"
                :default-upload="false"
                :show-file-list="false"
                :accept="kind === 'amxx' ? '.amxx,.sma' : '.so,.dll'"
                @change="onUploadChange"
            >
                <n-upload-dragger>
                    <div class="flex flex-col items-center gap-2 py-6">
                        <GIcon name="upload" class="text-4xl text-stone-400" />
                        <p class="text-stone-700 dark:text-stone-300 font-medium">
                            {{ trans('drop_hint') }}
                        </p>
                        <p class="text-sm text-stone-500 dark:text-stone-500">
                            {{ trans(kind === 'amxx' ? 'file_hint_amxx' : 'file_hint_metamod') }}
                        </p>
                    </div>
                </n-upload-dragger>
            </n-upload>

            <template v-else>
                <div
                    class="flex items-center gap-3 p-3 rounded border border-stone-200 dark:border-stone-700 bg-stone-50 dark:bg-stone-900"
                >
                    <GIcon name="file-code" size="lg" class="text-stone-400" />
                    <div class="min-w-0 flex-1">
                        <div class="font-mono text-sm text-stone-800 dark:text-stone-100 truncate">
                            {{ file.name }}
                        </div>
                        <div class="text-xs text-stone-400">{{ prettySize }}</div>
                    </div>
                    <button
                        v-if="!uploading"
                        class="text-stone-400 hover:text-stone-600 dark:hover:text-stone-200"
                        @click="file = null"
                    >
                        <GIcon name="xmark" />
                    </button>
                </div>

                <n-progress
                    v-if="uploading"
                    type="line"
                    :percentage="progress"
                    :show-indicator="false"
                    :height="8"
                    :border-radius="4"
                    processing
                />
            </template>

            <n-alert v-if="validationError" type="warning" :show-icon="true">
                {{ validationError }}
            </n-alert>

            <div v-if="compileErrors.length">
                <n-alert type="error" :show-icon="true">
                    {{ trans('compile_failed') }}
                </n-alert>
                <ul class="mt-2 space-y-1 text-xs font-mono">
                    <li
                        v-for="(diag, idx) in compileErrors"
                        :key="idx"
                        class="px-2 py-1 rounded bg-stone-50 dark:bg-stone-900 text-stone-700 dark:text-stone-200"
                    >
                        <span class="text-stone-400">{{ diag.severity === 'warning' ? 'W' : 'E' }}{{ diag.code }}</span>
                        <span class="mx-1 text-orange-500">:{{ diag.line }}</span>
                        {{ diag.message }}
                    </li>
                </ul>
            </div>

            <div
                v-if="isOverwrite && !validationError"
                class="border border-orange-300 dark:border-orange-800 rounded p-3 bg-orange-50 dark:bg-orange-950/40"
            >
                <div class="flex items-center gap-2">
                    <GIcon name="warning" class="text-orange-500" />
                    <strong class="text-orange-700 dark:text-orange-300">
                        {{ trans('overwrite_title') }}
                    </strong>
                </div>
                <p class="mt-1 text-sm text-orange-700 dark:text-orange-300">
                    {{ trans('overwrite_text') }}
                </p>
            </div>

            <div class="flex items-center justify-between flex-wrap gap-2">
                <n-checkbox :checked="autoEnable" @update:checked="(value: boolean) => (autoEnable = value)">
                    {{ trans('auto_enable') }}
                </n-checkbox>
                <div class="text-xs text-stone-400 dark:text-stone-500 font-mono">→ {{ targetPath }}</div>
            </div>
        </div>

        <template #footer>
            <GButton
                :color="isOverwrite ? 'orange' : 'green'"
                :disabled="!file || Boolean(validationError) || uploading"
                @click="install"
            >
                <GIcon :name="uploading ? 'spinner' : isOverwrite ? 'refresh' : 'download'" class="mr-1" />
                {{
                    uploading
                        ? trans('uploading')
                        : isOverwrite
                          ? trans('overwrite')
                          : trans('install')
                }}
            </GButton>
        </template>
    </GModal>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { NAlert, NCheckbox, NProgress, NUpload, NUploadDragger } from 'naive-ui';
import type { UploadFileInfo } from 'naive-ui';
import { usePluginTrans } from '@gameap/plugin-sdk';

import { fmEnsureDirectory, fmUploadFile } from '../api/gameap';
import { apiErrorMessage, compileSource, registerPlugin } from '../api/plugin';
import { fileExtension, fileStem, metamodDirName, prettyName } from '../lib/naming';
import type { CompileDiagnostic, PlatformKind, StatePaths } from '../types';

const props = defineProps<{
    show: boolean;
    kind: PlatformKind;
    serverId: number;
    pluginId: string;
    paths: StatePaths;
    existingFiles: string[];
}>();

const emit = defineEmits<{
    'update:show': [value: boolean];
    installed: [replaced: boolean];
}>();

const { trans } = usePluginTrans();

const file = ref<File | null>(null);
const autoEnable = ref(true);
const uploading = ref(false);
const progress = ref(0);
const compileErrors = ref<CompileDiagnostic[]>([]);

watch(
    () => props.show,
    (shown) => {
        if (shown) {
            file.value = null;
            autoEnable.value = true;
            uploading.value = false;
            progress.value = 0;
            compileErrors.value = [];
        }
    },
);

const validationError = computed(() => {
    if (!file.value) {
        return null;
    }
    const ext = fileExtension(file.value.name);
    if (props.kind === 'amxx') {
        return ext === 'amxx' || ext === 'sma' ? null : trans('wrong_type_amxx');
    }
    return ext === 'so' || ext === 'dll' ? null : trans('wrong_type_metamod');
});

/** .sma sources compile into <stem>.amxx. */
const isSource = computed(() =>
    props.kind === 'amxx' && file.value !== null && fileExtension(file.value.name) === 'sma',
);

/** The picked file matches an already registered plugin. */
const isOverwrite = computed(() => {
    if (!file.value) {
        return false;
    }
    // A source install overwrites the compiled plugin with the same stem.
    const name = isSource.value
        ? `${fileStem(file.value.name)}.amxx`.toLowerCase()
        : file.value.name.toLowerCase();
    return props.existingFiles.some((existing) => existing.toLowerCase() === name);
});

/** "cstrike/addons" — derived from the amxx dir path. */
const addonsRoot = computed(() => {
    const dir = props.paths.amxx_dir;
    const idx = dir.lastIndexOf('/');
    return idx > 0 ? dir.slice(0, idx) : dir;
});

const targetPath = computed(() => {
    if (props.kind === 'amxx') {
        return isSource.value
            ? `${props.paths.amxx_scripting_dir}/`
            : `${props.paths.amxx_plugins_dir}/`;
    }
    const name = file.value ? metamodDirName(file.value.name) : '…';
    return `${addonsRoot.value}/${name}/`;
});

const prettySize = computed(() => {
    if (!file.value) {
        return '';
    }
    const size = file.value.size;
    if (size < 1024) {
        return `${size} B`;
    }
    if (size < 1024 * 1024) {
        return `${Math.round(size / 1024)} KB`;
    }
    return `${(size / 1024 / 1024).toFixed(1)} MB`;
});

function onUploadChange(payload: { file: UploadFileInfo }): void {
    file.value = payload.file.file ?? null;
}

function onUpdateShow(value: boolean): void {
    if (!uploading.value) {
        emit('update:show', value);
    }
}

async function install(): Promise<void> {
    const picked = file.value;
    if (!picked || validationError.value) {
        return;
    }
    const replaced = isOverwrite.value;
    uploading.value = true;
    progress.value = 0;
    compileErrors.value = [];
    try {
        if (isSource.value) {
            await installFromSource(picked, replaced);
        } else if (props.kind === 'amxx') {
            await fmUploadFile(props.serverId, props.paths.amxx_plugins_dir, picked, (percent) => {
                progress.value = percent;
            });
            await registerPlugin(props.pluginId, props.serverId, 'amxx', {
                file: picked.name,
                enable: autoEnable.value,
                force: replaced,
            });
        } else {
            const dirName = metamodDirName(picked.name);
            await fmEnsureDirectory(props.serverId, addonsRoot.value, dirName);
            await fmUploadFile(props.serverId, `${addonsRoot.value}/${dirName}`, picked, (percent) => {
                progress.value = percent;
            });
            await registerPlugin(props.pluginId, props.serverId, 'metamod', {
                file: picked.name,
                enable: autoEnable.value,
                path: `addons/${dirName}/${picked.name}`,
                force: replaced,
            });
        }
        window.$message?.success(
            trans(replaced ? 'updated_toast' : 'installed_toast', { name: prettyName(picked.name) }),
        );
        emit('installed', replaced);
        emit('update:show', false);
    } catch (error) {
        window.$message?.error(apiErrorMessage(error, trans('op_failed')));
    } finally {
        uploading.value = false;
    }
}

/** Uploads a .sma into scripting/, compiles it and registers the result. */
async function installFromSource(picked: File, replaced: boolean): Promise<void> {
    await fmUploadFile(props.serverId, props.paths.amxx_scripting_dir, picked, (percent) => {
        progress.value = percent;
    });
    const result = await compileSource(props.pluginId, props.serverId, picked.name);
    if (!result.success || !result.amxx_file) {
        // The source stays in scripting/ so it can be fixed and recompiled.
        compileErrors.value = result.diagnostics;
        throw new Error(trans('compile_failed'));
    }
    await registerPlugin(props.pluginId, props.serverId, 'amxx', {
        file: result.amxx_file,
        enable: autoEnable.value,
        force: replaced,
    });
}
</script>
