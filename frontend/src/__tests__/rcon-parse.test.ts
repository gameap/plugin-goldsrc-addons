import { describe, expect, it } from 'vitest';

import {
    matchesListedFile,
    parseAmxxPlugins,
    parseAmxxVersion,
    parseMetaList,
    parseMetaVersion,
} from '../lib/rcon-parse';

describe('parseMetaVersion', () => {
    it('parses metamod-r', () => {
        const output = [
            'Metamod-r v1.3.0.149, API (5:13)',
            'Metamod-r build: 17:47:54 Jul 22 2018',
            'Metamod-r from: https://github.com/theAsmodai/metamod-r/',
        ].join('\n');
        expect(parseMetaVersion(output)).toEqual({ build: 'Metamod-r', version: '1.3.0.149' });
    });

    it('parses metamod-p and classic metamod', () => {
        expect(parseMetaVersion('Metamod-p v1.21p38  2013/05/30')).toEqual({
            build: 'Metamod-P',
            version: '1.21p38',
        });
        expect(parseMetaVersion('Metamod v1.20  2013/05/30 (5:13)')).toEqual({
            build: 'Metamod',
            version: '1.20',
        });
    });

    it('returns null when metamod is not present', () => {
        expect(parseMetaVersion('unknown command: meta')).toBeNull();
        expect(parseMetaVersion('')).toBeNull();
    });
});

describe('parseAmxxVersion', () => {
    it('parses the version banner', () => {
        const output = [
            'AMX Mod X 1.9.0.5294 (http://www.amxmodx.org)',
            'Authors:',
            '        David "BAILOPAN" Anderson, ...',
        ].join('\n');
        expect(parseAmxxVersion(output)).toEqual({ build: 'AMX Mod X', version: '1.9.0.5294' });
    });

    it('returns null when amxx is not present', () => {
        expect(parseAmxxVersion('unknown command: amxx')).toBeNull();
    });
});

describe('parseMetaList', () => {
    const output = [
        'Currently loaded plugins:',
        '      description      stat pend  file              vers      src   load  unlod',
        ' [ 1] AMX Mod X        RUN   -    amxmodx_mm_i386.  v1.9.0.5  ini   Start ANY',
        ' [ 2] Reunion          RUN   -    reunion_mm_i386.  v0.1.92   ini   Start Never',
        ' [ 3] WHBlocker        badf  -    whblocker_mm_i3   v1.5.697  ini   Start Never',
        '3 plugins, 2 running',
    ].join('\n');

    it('parses entries with statuses', () => {
        const list = parseMetaList(output);
        expect(list).toHaveLength(3);
        expect(list[0]).toMatchObject({
            name: 'AMX Mod X',
            file: 'amxmodx_mm_i386.',
            status: 'running',
            version: '1.9.0.5',
        });
        expect(list[2]).toMatchObject({ name: 'WHBlocker', status: 'error', rawStatus: 'badf' });
    });

    it('ignores non-entry lines', () => {
        expect(parseMetaList('Currently loaded plugins:\nno entries')).toHaveLength(0);
    });
});

describe('parseAmxxPlugins', () => {
    const output = [
        'Currently loaded plugins:',
        '       name                    version  author            file             status',
        '[  1] Admin Base              1.9.0.52 AMXX Dev Team     admin.amxx       running',
        '[  2] StatsX                  1.9.0.52 AMXX Dev Team     statsx.amxx      running',
        '[  3] CSDM Main               2.1.3d   BAILOPAN          csdm_main.amxx   bad load',
        '[  4] Parachute               1.3      KRoTaL            parachute.amxx   debug',
        '4 plugins, 3 running',
    ].join('\n');

    it('parses entries', () => {
        const list = parseAmxxPlugins(output);
        expect(list).toHaveLength(4);
        expect(list[0]).toMatchObject({
            name: 'Admin Base',
            version: '1.9.0.52',
            author: 'AMXX Dev Team',
            file: 'admin.amxx',
            status: 'running',
        });
        expect(list[2]).toMatchObject({ status: 'error', rawStatus: 'bad load' });
        expect(list[3]).toMatchObject({ status: 'running' });
    });
});

describe('matchesListedFile', () => {
    it('matches exact and truncated names', () => {
        expect(matchesListedFile('admin.amxx', 'admin.amxx')).toBe(true);
        expect(matchesListedFile('amxmodx_mm_i386.', 'amxmodx_mm_i386.so')).toBe(true);
        expect(matchesListedFile('whblocker_mm_i3', 'whblocker_mm_i386.so')).toBe(true);
        expect(matchesListedFile('admin.amxx', 'admincmd.amxx')).toBe(false);
        expect(matchesListedFile('a.', 'ab.amxx')).toBe(false);
    });
});
