import { describe, expect, it } from 'vitest';

import { computeRowStatus, isPendingRow, pauseActionForStatus } from '../lib/status';
import type { RowStatus, RuntimePluginInfo } from '../types';

function runtime(status: RuntimePluginInfo['status'], rawStatus: string = status): RuntimePluginInfo {
    return {
        file: 'test.amxx',
        name: 'Test Plugin',
        version: '1.0',
        author: 'Author',
        status,
        rawStatus,
    };
}

describe('computeRowStatus', () => {
    it('reports missing regardless of runtime and rcon', () => {
        expect(computeRowStatus({ enabled: true, missing: true, runtime: runtime('running'), rconOk: true }))
            .toEqual({ status: 'missing', detail: null });
        expect(computeRowStatus({ enabled: false, missing: true, runtime: null, rconOk: false }))
            .toEqual({ status: 'missing', detail: null });
    });

    it('reports runtime errors with the raw status as detail', () => {
        expect(computeRowStatus({ enabled: true, missing: false, runtime: runtime('error', 'bad load'), rconOk: true }))
            .toEqual({ status: 'error', detail: 'bad load' });
        expect(computeRowStatus({ enabled: false, missing: false, runtime: runtime('error', 'badf'), rconOk: false }))
            .toEqual({ status: 'error', detail: 'badf' });
    });

    describe('without rcon', () => {
        it('falls back to the ini flag', () => {
            expect(computeRowStatus({ enabled: true, missing: false, runtime: null, rconOk: false }))
                .toEqual({ status: 'enabled', detail: null });
            expect(computeRowStatus({ enabled: false, missing: false, runtime: null, rconOk: false }))
                .toEqual({ status: 'stopped', detail: null });
        });
    });

    describe('with rcon', () => {
        it('enabled + running → running', () => {
            expect(computeRowStatus({ enabled: true, missing: false, runtime: runtime('running'), rconOk: true }))
                .toEqual({ status: 'running', detail: null });
        });

        it('enabled + paused → paused, not pending', () => {
            expect(computeRowStatus({ enabled: true, missing: false, runtime: runtime('paused'), rconOk: true }))
                .toEqual({ status: 'paused', detail: 'paused' });
        });

        it('enabled + stopped → stopped, not pending', () => {
            expect(computeRowStatus({ enabled: true, missing: false, runtime: runtime('stopped'), rconOk: true }))
                .toEqual({ status: 'stopped', detail: 'stopped' });
        });

        it('disabled + stopped → stopped, not pending', () => {
            expect(computeRowStatus({ enabled: false, missing: false, runtime: runtime('stopped'), rconOk: true }))
                .toEqual({ status: 'stopped', detail: 'stopped' });
        });

        it('enabled + not loaded → pending', () => {
            expect(computeRowStatus({ enabled: true, missing: false, runtime: null, rconOk: true }))
                .toEqual({ status: 'pending', detail: null });
        });

        it('disabled + still in memory → pending', () => {
            expect(computeRowStatus({ enabled: false, missing: false, runtime: runtime('running'), rconOk: true }))
                .toEqual({ status: 'pending', detail: null });
            expect(computeRowStatus({ enabled: false, missing: false, runtime: runtime('paused'), rconOk: true }))
                .toEqual({ status: 'pending', detail: null });
        });

        it('disabled + not loaded → stopped', () => {
            expect(computeRowStatus({ enabled: false, missing: false, runtime: null, rconOk: true }))
                .toEqual({ status: 'stopped', detail: null });
        });
    });
});

describe('isPendingRow', () => {
    it('matches only the pending status', () => {
        const statuses: RowStatus[] = ['running', 'enabled', 'stopped', 'paused', 'pending', 'error', 'missing'];
        for (const status of statuses) {
            expect(isPendingRow(status)).toBe(status === 'pending');
        }
    });
});

describe('pauseActionForStatus', () => {
    it('offers pause for a running plugin', () => {
        expect(pauseActionForStatus('running')).toBe('pause');
    });

    it('offers unpause for a paused plugin', () => {
        expect(pauseActionForStatus('paused')).toBe('unpause');
    });

    it('offers no action for the other statuses', () => {
        const statuses: RowStatus[] = ['enabled', 'stopped', 'pending', 'error', 'missing'];
        for (const status of statuses) {
            expect(pauseActionForStatus(status)).toBeNull();
        }
    });
});
