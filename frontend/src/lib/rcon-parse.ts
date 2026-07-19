// Parsers of GoldSource console command output obtained via RCON.

import type { PlatformVersion, RuntimePluginInfo } from '../types';

/**
 * `meta version` →
 *   Metamod-r v1.3.0.149, API (5:13)
 *   Metamod-p v1.21p38 ...
 *   Metamod v1.21  2013/05/30 (5:13)
 */
export function parseMetaVersion(output: string): PlatformVersion | null {
    const match = /\bMetamod(-[a-z])?[ \t]+v?([0-9][\w.\-]*)/i.exec(output);
    if (!match) {
        return null;
    }
    const suffix = (match[1] ?? '').toLowerCase();
    const build = suffix === '-r' ? 'Metamod-r' : suffix === '-p' ? 'Metamod-P' : 'Metamod';
    return { build, version: match[2].replace(/[.,]+$/, '') };
}

/**
 * `amxx version` →
 *   AMX Mod X 1.9.0.5294 (http://www.amxmodx.org)
 */
export function parseAmxxVersion(output: string): PlatformVersion | null {
    const match = /AMX Mod X[ \t]+v?([0-9][\w.\-]*)/i.exec(output);
    if (!match) {
        return null;
    }
    return { build: 'AMX Mod X', version: match[1].replace(/[.,]+$/, '') };
}

/**
 * `meta list` table:
 *        description      stat pend  file              vers      src   load  unlod
 *   [ 1] AMX Mod X        RUN   -    amxmodx_mm_i386.  v1.9.0.5  ini   Start ANY
 *
 * Columns are fixed-width (a full column leaves a single space), so entries
 * are matched by anchoring on the known `stat` values rather than by
 * splitting on whitespace runs. File names arrive truncated.
 */
const META_LIST_LINE =
    /^\s*\[\s*\d+\]\s+(?<name>.+?)\s+(?<stat>RUN|PAUSE|badf|fail)\s+(?<pend>\S+)\s+(?<file>\S+)\s+v?(?<vers>\S+)/;

export function parseMetaList(output: string): RuntimePluginInfo[] {
    const result: RuntimePluginInfo[] = [];
    for (const line of output.split('\n')) {
        const match = META_LIST_LINE.exec(line.trimEnd());
        if (!match?.groups) {
            continue;
        }
        const { name, stat, file, vers } = match.groups;
        result.push({
            file,
            name: name.trim(),
            version: normalizeVersion(vers),
            author: null,
            status: normalizeMetaStatus(stat),
            rawStatus: stat,
        });
    }
    return result;
}

/**
 * `amxx plugins` table:
 *          name                    version  author            file             status
 *   [  1] Admin Base              1.9.0.52 AMXX Dev Team     admin.amxx       running
 *   [ 11] CSDM Main               2.1.3d   BAILOPAN          csdm_main.amxx   bad load
 *
 * Anchored on the known trailing status; name/author may contain spaces.
 */
const AMXX_PLUGINS_LINE =
    /^\s*\[\s*\d+\]\s+(?<name>.+?)\s+(?<version>v?\d\S*)\s+(?<author>.+?)\s+(?<file>\S+)\s+(?<status>running|debug|stopped|paused|bad load|error)\s*$/;

export function parseAmxxPlugins(output: string): RuntimePluginInfo[] {
    const result: RuntimePluginInfo[] = [];
    for (const line of output.split('\n')) {
        const match = AMXX_PLUGINS_LINE.exec(line.trimEnd());
        if (!match?.groups) {
            continue;
        }
        const { name, version, author, file, status } = match.groups;
        result.push({
            file,
            name: name.trim(),
            version: normalizeVersion(version),
            author: author.trim(),
            status: normalizeAmxxStatus(status),
            rawStatus: status,
        });
    }
    return result;
}

/**
 * Matches a (possibly truncated) file name from console output against the
 * real file name from plugins.ini.
 */
export function matchesListedFile(listed: string, actual: string): boolean {
    if (listed === actual) {
        return true;
    }
    const bare = listed.endsWith('.') ? listed.slice(0, -1) : listed;
    return bare.length >= 8 && actual.startsWith(bare);
}

function normalizeVersion(raw: string): string | null {
    const cleaned = raw.replace(/^v/i, '').trim();
    return /^[0-9]/.test(cleaned) ? cleaned : null;
}

function normalizeMetaStatus(stat: string): RuntimePluginInfo['status'] {
    const lowered = stat.toLowerCase();
    if (lowered === 'run') {
        return 'running';
    }
    if (lowered === 'pause') {
        return 'paused';
    }
    return 'error'; // badf, fail, ...
}

function normalizeAmxxStatus(status: string): RuntimePluginInfo['status'] {
    const lowered = status.toLowerCase();
    if (lowered === 'running' || lowered === 'debug') {
        return 'running';
    }
    if (lowered === 'paused' || lowered === 'stopped') {
        return 'paused';
    }
    return 'error'; // bad load, error
}
