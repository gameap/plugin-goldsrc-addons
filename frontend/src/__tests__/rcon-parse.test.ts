import { describe, expect, it } from 'vitest';

import {
    isBadPasswordOutput,
    matchesListedFile,
    parseAmxxPlugins,
    parseAmxxVersion,
    parseMetaList,
    parseMetaVersion,
    parseStatusMap,
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

    it('keeps stopped and paused as distinct statuses', () => {
        const list = parseAmxxPlugins(
            [
                '[  1] Admin Base              1.9.0.52 AMXX Dev Team     admin.amxx       paused',
                '[  2] CSDM Main               2.1.3d   BAILOPAN          csdm_main.amxx   stopped',
            ].join('\n'),
        );
        expect(list).toHaveLength(2);
        expect(list[0]).toMatchObject({ status: 'paused', rawStatus: 'paused' });
        expect(list[1]).toMatchObject({ status: 'stopped', rawStatus: 'stopped' });
    });

    it('parses the AMXX 1.10 table with id and url columns', () => {
        const list = parseAmxxPlugins(
            [
                'Currently loaded plugins:',
                '       id  name                    version     author            url                              file             status',
                '[  4] 3   FreshBans               1.4.8b      kanagava          unknown                          fresh_bans.  running',
                '[  5] 4   Admin Base              1.10.0.54   AMXX Dev Team     http://www.amxmodx.org           admin.amxx       stopped',
                '[  6] 5   Parachute               1.3         KRoTaL            unknown                          parachute.amxx   paused',
                '3 plugins, 1 running',
            ].join('\n'),
        );
        expect(list).toHaveLength(3);
        // The id must not stick to the name, the url must not stick to the author.
        expect(list[0]).toMatchObject({
            name: 'FreshBans',
            version: '1.4.8b',
            author: 'kanagava',
            file: 'fresh_bans.',
            status: 'running',
        });
        expect(list[1]).toMatchObject({
            name: 'Admin Base',
            author: 'AMXX Dev Team',
            file: 'admin.amxx',
            status: 'stopped',
        });
        expect(list[2]).toMatchObject({ name: 'Parachute', author: 'KRoTaL', status: 'paused' });
    });

    it('keeps a version suffix that spills into the author column', () => {
        const list = parseAmxxPlugins(
            '[ 19] Half-Life GunGame     2.3 Dev     serfreeman1337    gungame.amxx     debug',
        );
        expect(list).toHaveLength(1);
        expect(list[0]).toMatchObject({
            name: 'Half-Life GunGame',
            version: '2.3',
            author: 'Dev serfreeman1337',
            file: 'gungame.amxx',
            status: 'running',
        });
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

describe('parseStatusMap', () => {
    it('parses the current map', () => {
        const output = [
            'hostname:  Test Server',
            'version : 48/1.1.2.7/Stdio 8684 secure (10)',
            'tcp/ip  : 192.168.1.2:27015',
            'map     : de_dust2 at: 0 x, 0 y, 0 z',
            'players : 3 active (32 max)',
        ].join('\n');
        expect(parseStatusMap(output)).toBe('de_dust2');
    });

    it('returns null on garbage', () => {
        expect(parseStatusMap('unknown command: status')).toBeNull();
        expect(parseStatusMap('')).toBeNull();
    });
});

describe('isBadPasswordOutput', () => {
    it('matches the wrong-password answer in any case', () => {
        expect(isBadPasswordOutput('Bad Password')).toBe(true);
        expect(isBadPasswordOutput('bad rcon password')).toBe(true);
        expect(isBadPasswordOutput('BAD PASSWORD')).toBe(true);
        expect(isBadPasswordOutput('Bad Rcon Password')).toBe(true);
    });

    it('matches despite surrounding whitespace and newlines', () => {
        expect(isBadPasswordOutput('  Bad Password\n')).toBe(true);
        expect(isBadPasswordOutput('\nBad Password  \n')).toBe(true);
    });

    it('does not match regular command output', () => {
        const pluginsRow =
            '[  1] Admin Base              1.9.0.52 AMXX Dev Team     admin.amxx       running';
        expect(isBadPasswordOutput(pluginsRow)).toBe(false);
        expect(isBadPasswordOutput('unknown command: amxx')).toBe(false);
        expect(isBadPasswordOutput('')).toBe(false);
    });
});
