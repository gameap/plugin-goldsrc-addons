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
 * `amxx plugins` tables are fixed-width: printf truncates every field to its
 * column width, so splitting on whitespace mangles values that contain spaces
 * (a `2.3 Dev` version is a single column). Column offsets are derived from
 * the table header when it is present; without a header, hardcoded layouts
 * from the AMXX sources are used (amxmodx/srvcmd.cpp, `amx_command`):
 *
 *   1.9 branch:  " [%3d] %-23.22s %-11.10s %-17.16s %-16.15s %-9.8s\n"
 *     → name=7  version=31 author=43 file=61 status=78
 *   1.10 branch adds id and url columns:
 *                " [%3d] %-3i %-23.22s %-11.10s %-17.16s %-32.31s %-12.11s %-9.8s\n"
 *     → id=7 name=11 version=35 author=47 url=65 file=98 status=111
 *
 * Statuses come from CPlugin::getStatus() (amxmodx/CPlugin.cpp): running,
 * debug, paused, bad load, stopped, error ("locked" is ps_locked, UNUSED).
 */
interface AmxxPluginsLayout {
    name: number;
    version: number;
    author: number;
    /** AMXX 1.10 only: the author column ends where the url column starts. */
    url: number | null;
    file: number;
    status: number;
}

const AMXX_PLUGINS_LAYOUT_19: AmxxPluginsLayout = {
    name: 7,
    version: 31,
    author: 43,
    url: null,
    file: 61,
    status: 78,
};

const AMXX_PLUGINS_LAYOUT_110: AmxxPluginsLayout = {
    name: 11,
    version: 35,
    author: 47,
    url: 65,
    file: 98,
    status: 111,
};

const AMXX_PLUGINS_ROW = /^\s*\[\s*\d+\]/;
const AMXX_PLUGINS_ROW_WITH_ID = /^\s*\[\s*\d+\]\s+\d+\s/;
const AMXX_PLUGINS_HEADER = /^\s*(?:id\s+)?name\s+version\s+author\s+(?:url\s+)?file\s+status\s*$/;
const AMXX_PLUGIN_STATUSES = new Set(['running', 'debug', 'paused', 'stopped', 'bad load', 'error']);

function headerColumnOffset(line: string, word: string): number {
    const match = new RegExp(`\\b${word}\\b`).exec(line);
    return match ? match.index : -1;
}

/**
 * Locates the table header and derives column offsets from it, so the parser
 * follows the exact column widths of whichever AMXX build produced the
 * output. Returns null when there is no usable header line.
 */
function amxxPluginsHeaderLayout(lines: string[]): AmxxPluginsLayout | null {
    for (const line of lines) {
        if (!AMXX_PLUGINS_HEADER.test(line)) {
            continue;
        }
        const [name, version, author, url, file, status] = [
            'name',
            'version',
            'author',
            'url',
            'file',
            'status',
        ].map((word) => headerColumnOffset(line, word));
        const chain = url >= 0 ? [name, version, author, url, file, status] : [name, version, author, file, status];
        if (chain.some((offset) => offset < 0) || !chain.every((offset, i) => i === 0 || chain[i - 1] < offset)) {
            continue;
        }
        return { name, version, author, url: url >= 0 ? url : null, file, status };
    }
    return null;
}

export function parseAmxxPlugins(output: string): RuntimePluginInfo[] {
    const lines = output.split('\n');
    const headerLayout = amxxPluginsHeaderLayout(lines);
    const result: RuntimePluginInfo[] = [];

    for (const rawLine of lines) {
        if (!AMXX_PLUGINS_ROW.test(rawLine)) {
            continue;
        }
        // Rows occasionally lose their leading space when the console stream
        // is captured; column offsets assume the canonical " [ N] ..." shape.
        const line = ` ${rawLine.trimStart()}`.trimEnd();
        const layout =
            headerLayout ??
            (AMXX_PLUGINS_ROW_WITH_ID.test(line) ? AMXX_PLUGINS_LAYOUT_110 : AMXX_PLUGINS_LAYOUT_19);

        const status = line.slice(layout.status).trim();
        if (!AMXX_PLUGIN_STATUSES.has(status)) {
            continue; // truncated or merged row
        }
        result.push({
            file: line.slice(layout.file, layout.status).trim(),
            name: line.slice(layout.name, layout.version).trim(),
            version: normalizeVersion(line.slice(layout.version, layout.author).trim()),
            author: line.slice(layout.author, layout.url ?? layout.file).trim() || null,
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

/**
 * Pairs plugins.ini file names with runtime entries from console output.
 * plugins.ini defines the load order, so `amxx plugins` / `meta list` print
 * plugins in the same sequence; consuming each runtime entry at most once
 * keeps duplicate truncated names (the file column is width-limited, e.g.
 * `chatmanager` listed three times) aligned with their own ini rows instead
 * of every row matching the first runtime entry.
 *
 * When a duplicate group has fewer runtime entries than ini files (a sibling
 * is not loaded), the truncated names are indistinguishable — later ini files
 * simply get null, which is no worse than a wrong first-match.
 */
export function matchRuntimeToFiles(
    files: string[],
    runtimeList: RuntimePluginInfo[],
): (RuntimePluginInfo | null)[] {
    const claimed = new Set<number>();
    return files.map((file) => {
        const index = runtimeList.findIndex(
            (item, i) => !claimed.has(i) && matchesListedFile(item.file, file),
        );
        if (index < 0) {
            return null;
        }
        claimed.add(index);
        return runtimeList[index];
    });
}

/**
 * `status` output:
 *   map     : de_dust2 at: 0 x, 0 y, 0 z
 */
export function parseStatusMap(output: string): string | null {
    const match = /^map\s+:\s*(\S+)/im.exec(output);
    return match ? match[1] : null;
}

/** HLDS answers "Bad Password" to a wrong-password rcon command. */
export function isBadPasswordOutput(output: string): boolean {
    return /bad\s*(rcon\s*)?password/i.test(output);
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
    if (lowered === 'paused') {
        return 'paused';
    }
    if (lowered === 'stopped') {
        return 'stopped';
    }
    return 'error'; // bad load, error
}
