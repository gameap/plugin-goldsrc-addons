import { describe, expect, it } from 'vitest';

import {
    isBadPasswordOutput,
    matchesListedFile,
    matchRuntimeToFiles,
    parseAmxxPlugins,
    parseAmxxVersion,
    parseMetaList,
    parseMetaVersion,
    parseStatusMap,
} from '../lib/rcon-parse';
import type { RuntimePluginInfo } from '../types';

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

/**
 * Fixture builders mirror the fixed-width printf formats of `amxx plugins`
 * from amxmodx/srvcmd.cpp (`amx_command`), so columns keep their real widths
 * instead of hand-aligned whitespace:
 *   1.9:  " [%3d] %-23.22s %-11.10s %-17.16s %-16.15s %-9.8s"
 *   1.10: " [%3d] %-3i %-23.22s %-11.10s %-17.16s %-32.31s %-12.11s %-9.8s"
 */
const col = (value: string, width: number, content: number): string =>
    value.slice(0, content).padEnd(width);

const amxx19Header = (): string =>
    `       ${col('name', 23, 22)} ${col('version', 11, 10)} ${col('author', 17, 16)} ${col('file', 16, 15)} ${col('status', 9, 8)}`;

const amxx19Row = (
    num: number,
    name: string,
    version: string,
    author: string,
    file: string,
    status: string,
): string =>
    ` [${String(num).padStart(3)}] ${col(name, 23, 22)} ${col(version, 11, 10)} ${col(author, 17, 16)} ${col(file, 16, 15)} ${col(status, 9, 8)}`;

const amxx110Header = (): string =>
    `       ${col('id', 3, 2)} ${col('name', 23, 22)} ${col('version', 11, 10)} ${col('author', 17, 16)} ${col('url', 32, 31)} ${col('file', 12, 11)} ${col('status', 9, 8)}`;

const amxx110Row = (
    num: number,
    id: number,
    name: string,
    version: string,
    author: string,
    url: string,
    file: string,
    status: string,
): string =>
    ` [${String(num).padStart(3)}] ${col(String(id), 3, 3)} ${col(name, 23, 22)} ${col(version, 11, 10)} ${col(author, 17, 16)} ${col(url, 32, 31)} ${col(file, 12, 11)} ${col(status, 9, 8)}`;

describe('parseAmxxPlugins', () => {
    it('parses the AMXX 1.9 table', () => {
        const output = [
            'Currently loaded plugins:',
            amxx19Header(),
            amxx19Row(1, 'Admin Base', '1.9.0.5294', 'AMXX Dev Team', 'admin.amxx', 'running'),
            amxx19Row(2, 'StatsX', '1.9.0.5294', 'AMXX Dev Team', 'statsx.amxx', 'running'),
            amxx19Row(3, 'CSDM Main', '2.1.3d', 'BAILOPAN', 'csdm_main.amxx', 'bad load'),
            amxx19Row(4, 'Parachute', '1.3', 'KRoTaL', 'parachute.amxx', 'debug'),
            '(  3) Load fails: Plugin uses an unknown function (name "csdm")',
            '4 plugins, 3 running',
        ].join('\n');

        const list = parseAmxxPlugins(output);
        expect(list).toHaveLength(4);
        expect(list[0]).toMatchObject({
            name: 'Admin Base',
            version: '1.9.0.5294',
            author: 'AMXX Dev Team',
            file: 'admin.amxx',
            status: 'running',
        });
        expect(list[2]).toMatchObject({ status: 'error', rawStatus: 'bad load' });
        expect(list[3]).toMatchObject({ status: 'running', rawStatus: 'debug' });
    });

    it('keeps stopped and paused as distinct statuses', () => {
        const list = parseAmxxPlugins(
            [
                amxx19Header(),
                amxx19Row(1, 'Admin Base', '1.9.0.5294', 'AMXX Dev Team', 'admin.amxx', 'paused'),
                amxx19Row(2, 'CSDM Main', '2.1.3d', 'BAILOPAN', 'csdm_main.amxx', 'stopped'),
            ].join('\n'),
        );
        expect(list).toHaveLength(2);
        expect(list[0]).toMatchObject({ status: 'paused', rawStatus: 'paused' });
        expect(list[1]).toMatchObject({ status: 'stopped', rawStatus: 'stopped' });
    });

    it('keeps a spaced version inside the version column (1.9)', () => {
        const output = [
            'Currently loaded plugins:',
            amxx19Header(),
            amxx19Row(19, 'Half-Life GunGame', '2.3 Dev', 'serfreeman1337', 'gungame.amxx', 'running'),
            '1 plugins, 1 running',
        ].join('\n');

        const list = parseAmxxPlugins(output);
        expect(list).toHaveLength(1);
        expect(list[0]).toMatchObject({
            name: 'Half-Life GunGame',
            version: '2.3 Dev',
            author: 'serfreeman1337',
            file: 'gungame.amxx',
        });
    });

    it('parses a headerless 1.9 row using the hardcoded layout', () => {
        const list = parseAmxxPlugins(
            '[ 19] Half-Life GunGame       2.3 Dev     serfreeman1337    gungame.amxx     debug',
        );
        expect(list).toHaveLength(1);
        expect(list[0]).toMatchObject({
            name: 'Half-Life GunGame',
            version: '2.3 Dev',
            author: 'serfreeman1337',
            file: 'gungame.amxx',
            status: 'running',
            rawStatus: 'debug',
        });
    });

    it('parses a headerless 1.10 row that lost its leading space', () => {
        const list = parseAmxxPlugins(
            '[ 32] 31  AES: Bonus CStrike      0.5.9.1 [R  serfreeman1337/s                                   aes_bonus_c  running',
        );
        expect(list).toHaveLength(1);
        expect(list[0]).toMatchObject({
            name: 'AES: Bonus CStrike',
            version: '0.5.9.1 [R',
            author: 'serfreeman1337/s',
            file: 'aes_bonus_c',
            status: 'running',
        });
    });

    it('skips rows with an unknown status', () => {
        expect(
            parseAmxxPlugins(amxx19Row(1, 'Admin Base', '1.9.0.5294', 'AMXX Dev Team', 'admin.amxx', 'junk')),
        ).toHaveLength(0);
    });

    it('parses the AMXX 1.10 table with id and url columns', () => {
        const output = [
            'Currently loaded plugins:',
            amxx110Header(),
            amxx110Row(4, 3, 'FreshBans', '1.4.8b', 'kanagava', 'unknown', 'fresh_bans.amxx', 'running'),
            amxx110Row(5, 4, 'Admin Base', '1.10.0.54', 'AMXX Dev Team', 'http://www.amxmodx.org', 'admin.amxx', 'stopped'),
            amxx110Row(6, 5, 'Parachute', '1.3', 'KRoTaL', 'unknown', 'parachute.amxx', 'paused'),
            '3 plugins, 1 running',
        ].join('\n');

        const list = parseAmxxPlugins(output);
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
});

describe('parseAmxxPlugins real-world AMXX 1.10 listing', () => {
    // Verbatim server capture: note the "[ 32]" row that lost its leading
    // space and the rows without trailing padding ("[ 31]", "[ 63]").
    const output = [
        'Currently loaded plugins:',
        '       id  name                    version     author            url                              file         status   ',
        ' [  1] 0   Guard: Core             1.0.8       mx?!                                               guard_core.  running  ',
        ' [  2] 1   Guard: IPHub-Client     1.3         mx?!                                               guard_iphub  running  ',
        ' [  3] 2   Guard: ReAimDetector A  0.2.2       ReHLDS Team                                        guard_reaim  running  ',
        ' [  4] 3   FreshBans               1.4.8b      kanagava          unknown                          fresh_bans.  running  ',
        ' [  5] 4   Admin Loader            3.5         neygomon                                           admin_load.  running  ',
        ' [  6] 5   Mod: Players            4.5         neugomon & Slove                                   mod_players  running  ',
        ' [  7] 6   Mod: Commands           1.3         AMXX Dev Team /                                    mod_cmd.amx  running  ',
        ' [  8] 7   Menu: Mod               2.1         Slove. & Gemini                                    menu_mod.am  running  ',
        ' [  9] 8   Menu: Server            1.0         Slove.                                             menu_server  running  ',
        ' [ 10] 9   Menu: Maps              1.5         neugomon & AcE /                                   menu_maps.a  running  ',
        ' [ 11] 10  Menu: MapChooser        2.7         neygomon & Gemin                                   menu_mapcho  running  ',
        ' [ 12] 11  Menu: ADMIN             2.6         Slove.                                             menu_admin.  running  ',
        ' [ 13] 12  Menu: PREMIUM           2.6         Slove.                                             menu_premiu  running  ',
        ' [ 14] 13  Menu: VIP               2.6         Slove.                                             menu_vip.am  running  ',
        ' [ 15] 14  Menu: Kabinet           1.3         Slove.                                             menu_kabine  running  ',
        ' [ 16] 15  Menu: Language          1.6         F@nt0M & Slove.                                    menu_lang.a  running  ',
        ' [ 17] 16  Menu: Damager           0.1.0       steelzzz & Alber                                   menu_damage  running  ',
        ' [ 18] 17  Menu: ScreenFade        0.0.9       Vaqtincha                                          menu_screen  running  ',
        ' [ 19] 18  Menu: Knife             1.0         Ragamafona & Slo                                   menu_knife.  running  ',
        ' [ 20] 19  Menu: Models            2.6         TheRedShoko & Ge                                   menu_models  running  ',
        ' [ 21] 20  Menu: Smoke             1.5.3       Slove.                                             menu_smoke.  running  ',
        ' [ 22] 21  Menu: Gag               1.5.2       neygomon                                           menu_gag.am  running  ',
        ' [ 23] 22  Menu: VoteBan           1.5.8       neygomon & Gemin                                   menu_voteba  running  ',
        ' [ 24] 23  CSStatsX SQL            0.7.4+2fix  serfreeman1337                                     csstatsx_sq  running  ',
        ' [ 25] 24  Stats Configuration     1.10.0.547  AMXX Dev Team                                      statscfg.am  running  ',
        ' [ 26] 25  AES: StatsX             0.5.9 [REA  serfreeman1337/s                                   statsx_db.a  running  ',
        ' [ 27] 26  AES: Core               0.5.9 [REA  serfreeman1337/s                                   aes_main.am  running  ',
        ' [ 28] 27  AES: CStrike Addon      0.5.9 [REA  serfreeman1337/s                                   aes_exp_cst  running  ',
        ' [ 29] 28  AES: Informer           0.5.9 [REA  serfreeman1337/s                                   aes_informe  running  ',
        ' [ 30] 29  AES: Admin Tools        0.5.9 [REA  serfreeman1337/s                                   aes_exp_edi  running  ',
        ' [ 31] 30  AES: Bonus System       0.5.9 Vega  serfreeman1337/s                                   aes_bonus_s  running',
        '[ 32] 31  AES: Bonus CStrike      0.5.9.1 [R  serfreeman1337/s                                   aes_bonus_c  running  ',
        ' [ 33] 32  AES: Bonus API          1.0.0       ArKaNeMaN                                          aes_bonus_a  running  ',
        ' [ 34] 33  AES: Coin               1.3         Nvoymax & Slove.                                   aes_bonus_c  running  ',
        ' [ 35] 34  Bonus: Privilege        1.0         Slove.                                             bonus_privi  running  ',
        ' [ 36] 35  Bonus: Holidays         0.0.4       Albertio & Slove                                   bonus_holid  running  ',
        ' [ 37] 36  Bonus: Client           1.1         Slove.                                             bonus_clien  running  ',
        ' [ 38] 37  Bonus: TOP Player       1.3.2       szawesome & Slov                                   bonus_top.a  running  ',
        ' [ 39] 38  Bonus: VIP Test         2.3         Javekson & Slove                                   bonus_vipte  running  ',
        ' [ 40] 39  Custom: Rcon Shop       0.6.1       b0t.                                               custom_rcon  running  ',
        ' [ 41] 40  Custom: Weapons         0.1.0b      steelzzz                                           custom_weap  running  ',
        ' [ 42] 41  Custom: Healthnade      0.0.19f     DEV-CS.RU Commun                                   custom_heal  running  ',
        ' [ 43] 42  Custom: Molotov         1.0.3       medusa                                             custom_molo  running  ',
        ' [ 44] 43  Custom: Mega Nade       1.6         mx?!                                               custom_mega  running  ',
        ' [ 45] 44  Custom: Server-Side Sm  2.1.0       Sergey Shorokhov                                   custom_smok  running  ',
        ' [ 46] 45  Control: AFK            1.5.5       neygomon & Slove                                   control_afk  running  ',
        ' [ 47] 46  Control: Ping           1.3         neygomon & Slove                                   control_pin  running  ',
        ' [ 48] 47  Control: Online         2.1.3       Nordic Warrior                                     control_onl  running  ',
        ' [ 49] 48  Control: Spawns         1.0.2       iPlague                                            control_spa  running  ',
        ' [ 50] 49  Control: Slots          1.4         pUzzlik & Slove.                                   control_slo  running  ',
        ' [ 51] 50  Control: Weapons        2.1         s1lent & neugomo                                   control_wea  running  ',
        ' [ 52] 51  Control: Nades          0.0.3a      steelzorrr                                         control_nad  running  ',
        ' [ 53] 52  Control: AWP            1.3.0 Beta  Nordic Warrior                                     control_awp  running  ',
        ' [ 54] 53  Control: Max Clip       1.0         fantom                                             control_max  running  ',
        ' [ 55] 54  Control: NickName Chan  1.4         sergrib                                            control_nic  running  ',
        ' [ 56] 55  Control: Logs           1.2.2       PAffAEJIkA                                         control_log  running  ',
        ' [ 57] 56  Game: Name              1.0         mx?!                                               game_name.a  running  ',
        ' [ 58] 57  Game: Timer             0.4         neygomon / mod b                                   game_timer.  running  ',
        ' [ 59] 58  Game: Warmup            2.8         neugomon/h1k3 &                                    game_warmup  running  ',
        ' [ 60] 59  Game: Resetscore        1.7         AcE & Slove.                                       game_resets  running  ',
        ' [ 61] 60  Game: Parachute         1.2         ReHLDS Team & Sl                                   game_parach  running  ',
        ' [ 62] 61  Game: Demorecord        1.8         neygomon & Slove                                   game_demore  running  ',
        ' [ 63] 62  Game: Mode              2.5re       s1lent                                             game_mode.a  running',
        ' [ 64] 63  Game: Decor             1.1         b0t.                                               game_decor.  running  ',
        ' [ 65] 64  Show: Online            0.2         Slove.                                             show_online  running  ',
        ' [ 66] 65  Show: Connected         3.1         Slove.                                             show_connec  running  ',
        ' [ 67] 66  Show: KillerHP          2.0         neygomon & Slove                                   show_killer  running  ',
        ' [ 68] 67  Show: FITH              1.1         CHEL74                                             show_fith.a  running  ',
        ' [ 69] 68  Show: Intro             1.2         Slove.                                             show_intro.  running  ',
        ' [ 70] 69  Winter: Environment     1.1         Slove.                                             winter_envi  stopped  ',
        ' [ 71] 70  Chat Manager: Advert    1.7         neygomon & Slove                                   chatmanager  running  ',
        ' [ 72] 71  Chat Manager: Core      1.1.3       Mistrick                                           chatmanager  running  ',
        ' [ 73] 72  Chat Manager: Guard     0.0.4-70    Mistrick                                           chatmanager  running  ',
        '73 plugins, 72 running',
    ].join('\n');

    const list = parseAmxxPlugins(output);

    it('parses every row, skipping header and summary lines', () => {
        expect(list).toHaveLength(73);
    });

    it('parses regular rows', () => {
        expect(list[0]).toMatchObject({
            name: 'Guard: Core',
            version: '1.0.8',
            author: 'mx?!',
            file: 'guard_core.',
            status: 'running',
        });
        expect(list[6]).toMatchObject({ name: 'Mod: Commands', author: 'AMXX Dev Team /', file: 'mod_cmd.amx' });
        expect(list[23]).toMatchObject({ name: 'CSStatsX SQL', version: '0.7.4+2fix', author: 'serfreeman1337' });
        expect(list[62]).toMatchObject({ name: 'Game: Mode', version: '2.5re', author: 's1lent' });
    });

    it('drops the url column from the author', () => {
        expect(list[3]).toMatchObject({ name: 'FreshBans', author: 'kanagava', file: 'fresh_bans.' });
    });

    it('keeps truncated versions in the version column, not in the author', () => {
        expect(list[25]).toMatchObject({
            name: 'AES: StatsX',
            version: '0.5.9 [REA',
            author: 'serfreeman1337/s',
        });
        expect(list[30]).toMatchObject({ name: 'AES: Bonus System', version: '0.5.9 Vega' });
        expect(list[52]).toMatchObject({
            name: 'Control: AWP',
            version: '1.3.0 Beta',
            author: 'Nordic Warrior',
        });
    });

    it('parses the row that lost its leading space', () => {
        expect(list[31]).toMatchObject({
            name: 'AES: Bonus CStrike',
            version: '0.5.9.1 [R',
            author: 'serfreeman1337/s',
            file: 'aes_bonus_c',
            status: 'running',
        });
    });

    it('keeps the stopped status', () => {
        expect(list[69]).toMatchObject({
            name: 'Winter: Environment',
            status: 'stopped',
            rawStatus: 'stopped',
        });
    });

    it('keeps duplicate truncated file names as separate entries', () => {
        expect(list[33]).toMatchObject({ name: 'AES: Coin', file: 'aes_bonus_c', version: '1.3' });
        expect(list[70]).toMatchObject({ name: 'Chat Manager: Advert', file: 'chatmanager', version: '1.7' });
        expect(list[71]).toMatchObject({ name: 'Chat Manager: Core', file: 'chatmanager', version: '1.1.3' });
        expect(list[72]).toMatchObject({ name: 'Chat Manager: Guard', file: 'chatmanager', version: '0.0.4-70' });
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

describe('matchRuntimeToFiles', () => {
    const rt = (file: string, name: string): RuntimePluginInfo => ({
        file,
        name,
        version: null,
        author: null,
        status: 'running',
        rawStatus: 'running',
    });

    it('matches unique names as before', () => {
        const matched = matchRuntimeToFiles(
            ['admin.amxx', 'statsx.amxx'],
            [rt('admin.amxx', 'Admin Base'), rt('statsx.amxx', 'StatsX')],
        );
        expect(matched.map((item) => item?.name)).toEqual(['Admin Base', 'StatsX']);
    });

    it('aligns duplicate truncated names by load order', () => {
        const runtime = [
            rt('aes_bonus_s', 'AES: Bonus System'),
            rt('aes_bonus_c', 'AES: Bonus CStrike'),
            rt('aes_bonus_a', 'AES: Bonus API'),
            rt('aes_bonus_c', 'AES: Coin'),
        ];
        const matched = matchRuntimeToFiles(
            ['aes_bonus_system.amxx', 'aes_bonus_cstrike.amxx', 'aes_bonus_api.amxx', 'aes_bonus_coin.amxx'],
            runtime,
        );
        expect(matched.map((item) => item?.name)).toEqual([
            'AES: Bonus System',
            'AES: Bonus CStrike',
            'AES: Bonus API',
            'AES: Coin',
        ]);
    });

    it('aligns three identical truncated names', () => {
        const runtime = [
            rt('chatmanager', 'Chat Manager: Advert'),
            rt('chatmanager', 'Chat Manager: Core'),
            rt('chatmanager', 'Chat Manager: Guard'),
        ];
        const matched = matchRuntimeToFiles(
            ['chatmanager_advert.amxx', 'chatmanager_core.amxx', 'chatmanager_guard.amxx'],
            runtime,
        );
        expect(matched.map((item) => item?.name)).toEqual([
            'Chat Manager: Advert',
            'Chat Manager: Core',
            'Chat Manager: Guard',
        ]);
    });

    it('consumes each runtime entry at most once', () => {
        const matched = matchRuntimeToFiles(
            ['chatmanager_advert.amxx', 'chatmanager_core.amxx'],
            [rt('chatmanager', 'Chat Manager: Advert')],
        );
        expect(matched[0]?.name).toBe('Chat Manager: Advert');
        expect(matched[1]).toBeNull();
    });

    it('returns null for ini files left without a runtime entry', () => {
        // A sibling is not loaded: truncated names are indistinguishable, so
        // later ini files get null instead of a wrong first-match.
        const matched = matchRuntimeToFiles(
            ['chatmanager_advert.amxx', 'chatmanager_core.amxx', 'chatmanager_guard.amxx'],
            [rt('chatmanager', 'Chat Manager: Advert'), rt('chatmanager', 'Chat Manager: Guard')],
        );
        expect(matched[0]?.name).toBe('Chat Manager: Advert');
        expect(matched[2]).toBeNull();
    });

    it('matches entries from the real 1.10 listing in load order', () => {
        const runtime = parseAmxxPlugins(
            [
                amxx110Header(),
                amxx110Row(31, 30, 'AES: Bonus System', '0.5.9 Vega', 'serfreeman1337/s', '', 'aes_bonus_s', 'running'),
                amxx110Row(32, 31, 'AES: Bonus CStrike', '0.5.9.1 [R', 'serfreeman1337/s', '', 'aes_bonus_c', 'running'),
                amxx110Row(33, 32, 'AES: Bonus API', '1.0.0', 'ArKaNeMaN', '', 'aes_bonus_a', 'running'),
                amxx110Row(34, 33, 'AES: Coin', '1.3', 'Nvoymax & Slove.', '', 'aes_bonus_c', 'running'),
                '4 plugins, 4 running',
            ].join('\n'),
        );
        const matched = matchRuntimeToFiles(
            ['aes_bonus_system.amxx', 'aes_bonus_cstrike.amxx', 'aes_bonus_api.amxx', 'aes_bonus_coin.amxx'],
            runtime,
        );
        expect(matched.map((item) => item?.name)).toEqual([
            'AES: Bonus System',
            'AES: Bonus CStrike',
            'AES: Bonus API',
            'AES: Coin',
        ]);
        expect(matched[3]?.version).toBe('1.3');
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
