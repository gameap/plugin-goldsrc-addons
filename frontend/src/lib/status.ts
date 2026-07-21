// Row status decision logic, kept pure so it can be unit-tested.

import type { RowStatus, RuntimePluginInfo } from '../types';

export interface RowStatusInput {
    enabled: boolean;
    missing: boolean;
    runtime: RuntimePluginInfo | null;
    rconOk: boolean;
}

export interface RowStatusResult {
    status: RowStatus;
    detail: string | null;
}

/**
 * Decides the row status from the ini state and the runtime (RCON) state.
 *
 * `paused` / `stopped` reached via `amxx pause` / `amxx stop` in the server
 * console are deliberate runtime states — they never become `pending` and
 * never ask for a restart.
 */
export function computeRowStatus({ enabled, missing, runtime, rconOk }: RowStatusInput): RowStatusResult {
    if (missing) {
        return { status: 'missing', detail: null };
    }
    if (runtime?.status === 'error') {
        return { status: 'error', detail: runtime.rawStatus };
    }
    if (!rconOk) {
        // Without console access there is no runtime to compare against.
        return { status: enabled ? 'enabled' : 'stopped', detail: null };
    }
    if (enabled && runtime?.status === 'running') {
        return { status: 'running', detail: null };
    }
    if (enabled && runtime?.status === 'paused') {
        return { status: 'paused', detail: runtime.rawStatus };
    }
    if (runtime?.status === 'stopped') {
        // Stopped in the game — no restart needed, regardless of the ini flag.
        return { status: 'stopped', detail: runtime.rawStatus };
    }
    if (enabled && runtime === null) {
        // Enabled in the ini but not loaded yet.
        return { status: 'pending', detail: null };
    }
    if (!enabled && runtime !== null) {
        // Disabled in the ini but still in memory.
        return { status: 'pending', detail: null };
    }
    return { status: enabled ? 'enabled' : 'stopped', detail: null };
}

export function isPendingRow(status: RowStatus): boolean {
    return status === 'pending';
}

/** Pause/unpause is a runtime action, available only against live console state. */
export function pauseActionForStatus(status: RowStatus): 'pause' | 'unpause' | null {
    if (status === 'running') return 'pause';
    if (status === 'paused') return 'unpause';
    return null;
}
