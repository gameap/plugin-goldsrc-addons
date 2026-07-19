// Client of the plugin's own WASM backend routes.

import axios from 'axios';

import type { PlatformKind, StateResponse } from '../types';

function base(pluginId: string): string {
    return `/api/plugins/${pluginId}`;
}

export async function getState(pluginId: string, serverId: number): Promise<StateResponse> {
    const response = await axios.get(`${base(pluginId)}/servers/${serverId}/state`);
    return response.data as StateResponse;
}

export async function togglePlugin(
    pluginId: string,
    serverId: number,
    platform: PlatformKind,
    file: string,
    enabled: boolean,
): Promise<void> {
    await axios.post(`${base(pluginId)}/servers/${serverId}/${platform}/plugins/toggle`, {
        file,
        enabled,
    });
}

export async function setAttributes(
    pluginId: string,
    serverId: number,
    platform: PlatformKind,
    file: string,
    debug: boolean,
    comment: string | null,
): Promise<void> {
    await axios.post(`${base(pluginId)}/servers/${serverId}/${platform}/plugins/attributes`, {
        file,
        debug,
        comment,
    });
}

export async function registerPlugin(
    pluginId: string,
    serverId: number,
    platform: PlatformKind,
    payload: { file: string; enable: boolean; path?: string; force?: boolean },
): Promise<void> {
    await axios.post(`${base(pluginId)}/servers/${serverId}/${platform}/plugins`, payload);
}

export async function deletePlugin(
    pluginId: string,
    serverId: number,
    platform: PlatformKind,
    file: string,
): Promise<void> {
    await axios.delete(`${base(pluginId)}/servers/${serverId}/${platform}/plugins`, {
        data: { file },
        params: { file },
    });
}

/** Human-oriented message from a backend error response. */
export function apiErrorMessage(error: unknown, fallback: string): string {
    if (axios.isAxiosError(error)) {
        const data = error.response?.data as { message?: string } | undefined;
        if (data?.message) {
            return data.message;
        }
        return error.message;
    }
    return fallback;
}
