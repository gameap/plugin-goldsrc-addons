// Pawn syntax highlighting for the .sma source editor.
//
// Prism has no official Pawn grammar, but Pawn is C-like: the C grammar
// (comments, strings, numbers, preprocessor) fits as-is, only the keyword
// set differs. Imports are limited to prism-core + clike + c (~10 KB min).

import Prism from 'prismjs/components/prism-core';
import 'prismjs/components/prism-clike';
import 'prismjs/components/prism-c';

Prism.languages.pawn = Prism.languages.extend('c', {
    keyword:
        /\b(?:assert|bool|break|case|char|const|continue|default|defined|do|else|enum|exit|Float|for|forward|goto|if|native|new|operator|public|return|sizeof|sleep|state|static|stock|switch|tagof|void|while)\b/,
    constant: /\b(?:cellmin|cellmax)\b/,
});

// C has no true/false keywords and prism-c drops `boolean`; Pawn has both.
// insertBefore keeps the token order sane (extend() would append at the end).
Prism.languages.insertBefore('pawn', 'function', {
    boolean: /\b(?:false|true)\b/,
});

/** Highlights Pawn source, returns HTML with Prism token spans. */
export function highlightPawn(code: string): string {
    return Prism.highlight(code, Prism.languages.pawn, 'pawn');
}

const STYLE_ID = 'goldsrc-addons-pawn-editor';

// Token colors plus gutter/caret colors. Injected from JS (not the SFC
// <style> block) because the plugin CSS bundle is not guaranteed to be
// applied by the panel. The editor is always dark — same look as the
// panel's file-manager text editor (VS Code Dark+ style).
const EDITOR_CSS = `
.pwn-editor { background: #1e1e1e; }
.pwn-editor .pwn-pre { color: #d4d4d4; }
.pwn-editor .token.comment { color: #6a9955; }
.pwn-editor .token.string, .pwn-editor .token.char { color: #ce9178; }
.pwn-editor .token.keyword, .pwn-editor .token.boolean { color: #569cd6; }
.pwn-editor .token.directive, .pwn-editor .token.directive-hash { color: #c586c0; }
.pwn-editor .token.number { color: #b5cea8; }
.pwn-editor .token.constant { color: #4fc1ff; }
.pwn-editor .token.function, .pwn-editor .token.macro-name { color: #dcdcaa; }
.pwn-editor .token.class-name { color: #4ec9b0; }
.pwn-editor .pwn-gutter { background: #252526; color: #858585; }
.pwn-editor .pwn-textarea { caret-color: #d4d4d4; }
/* Visible scrollbars on the dark background (VS Code Dark+ style); the
   default translucent dark thumb blends into #1e1e1e. */
.pwn-editor .pwn-textarea { scrollbar-color: rgba(121, 121, 121, 0.55) transparent; }
.pwn-editor .pwn-textarea::-webkit-scrollbar { width: 12px; height: 12px; }
.pwn-editor .pwn-textarea::-webkit-scrollbar-track { background: transparent; }
.pwn-editor .pwn-textarea::-webkit-scrollbar-corner { background: transparent; }
.pwn-editor .pwn-textarea::-webkit-scrollbar-thumb {
    background: rgba(121, 121, 121, 0.45);
    border-radius: 6px;
    border: 3px solid transparent;
    background-clip: padding-box;
}
.pwn-editor .pwn-textarea::-webkit-scrollbar-thumb:hover {
    background: rgba(160, 160, 160, 0.65);
    border: 3px solid transparent;
    background-clip: padding-box;
}
/* The selection paints in the textarea layer above the <pre>; it must stay
   translucent (and the glyphs transparent), otherwise it hides the
   highlighted text underneath. */
.pwn-editor .pwn-textarea::selection { background: rgba(38, 79, 120, 0.45); color: transparent; }
.pwn-editor .pwn-textarea::-moz-selection { background: rgba(38, 79, 120, 0.45); color: transparent; }
`;

/** Injects the editor's color styles into <head> once. Safe to call repeatedly. */
export function ensurePawnEditorStyles(): void {
    if (document.getElementById(STYLE_ID)) {
        return;
    }
    const style = document.createElement('style');
    style.id = STYLE_ID;
    style.textContent = EDITOR_CSS;
    document.head.appendChild(style);
}
