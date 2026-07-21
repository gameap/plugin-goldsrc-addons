import { describe, expect, it } from 'vitest';

import { highlightPawn } from '../lib/pawn';

describe('highlightPawn', () => {
    it('marks pawn keywords', () => {
        const html = highlightPawn('public plugin_init() { new bool:x = true; }');
        expect(html).toContain('<span class="token keyword">public</span>');
        expect(html).toContain('<span class="token keyword">new</span>');
        expect(html).toContain('<span class="token keyword">bool</span>');
        expect(html).toContain('<span class="token boolean">true</span>');
    });

    it('marks function calls', () => {
        expect(highlightPawn('plugin_init()')).toContain('<span class="token function">plugin_init</span>');
    });

    it('marks comments', () => {
        expect(highlightPawn('// note')).toContain('token comment');
        expect(highlightPawn('/* note */')).toContain('token comment');
    });

    it('marks #include as macro with the path as string', () => {
        const html = highlightPawn('#include <amxmodx>');
        expect(html).toContain('token macro');
        expect(html).toContain('token directive');
        expect(html).toContain('&lt;amxmodx>');
    });

    it('marks numbers and strings', () => {
        const html = highlightPawn('new x = 42; new s[] = "hi";');
        expect(html).toContain('<span class="token number">42</span>');
        expect(html).toContain('token string');
    });

    it('escapes html in source', () => {
        // Prism escapes `<` and `&`; a literal `>` in text content is valid
        // HTML and cannot start a tag, so it is left as-is.
        const html = highlightPawn('if (a < b && c > d)');
        expect(html).toContain('&lt;');
        expect(html).toContain('&amp;&amp;');
        expect(highlightPawn('new s[] = "<div>"')).not.toContain('<div>');
    });
});
