import { describe, expect, it } from 'vitest';

import { fileExtension, metamodDirName, prettyName } from '../lib/naming';

describe('metamodDirName', () => {
    it('strips metamod suffixes', () => {
        expect(metamodDirName('reunion_mm_i386.so')).toBe('reunion');
        expect(metamodDirName('podbot_mm.dll')).toBe('podbot');
        expect(metamodDirName('whblocker_mm_amd64.so')).toBe('whblocker');
        expect(metamodDirName('VoiceTranscoder.so')).toBe('VoiceTranscoder');
        expect(metamodDirName('rechecker_mm_i686.so')).toBe('rechecker');
    });
});

describe('fileExtension', () => {
    it('extracts lowercase extension', () => {
        expect(fileExtension('admin.AMXX')).toBe('amxx');
        expect(fileExtension('reunion.so')).toBe('so');
        expect(fileExtension('noext')).toBe('');
    });
});

describe('prettyName', () => {
    it('builds a readable name', () => {
        expect(prettyName('high_ping_kicker.amxx')).toBe('High Ping Kicker');
        expect(prettyName('csdm_main.amxx')).toBe('Csdm Main');
    });
});
