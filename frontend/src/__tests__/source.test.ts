import { describe, expect, it } from 'vitest';

import { lineOffset } from '../lib/source';

describe('lineOffset', () => {
    const text = '#include <amxmodx>\n\npublic plugin_init() {\n}\n';

    it('points at the start for line 1 and below', () => {
        expect(lineOffset(text, 1)).toBe(0);
        expect(lineOffset(text, 0)).toBe(0);
    });

    it('locates later lines', () => {
        expect(lineOffset(text, 2)).toBe(19);
        expect(lineOffset(text, 3)).toBe(20);
        expect(text[lineOffset(text, 3)]).toBe('p');
    });

    it('clamps past the end', () => {
        expect(lineOffset('a\nb', 5)).toBe(3);
        expect(lineOffset('', 3)).toBe(0);
    });
});
