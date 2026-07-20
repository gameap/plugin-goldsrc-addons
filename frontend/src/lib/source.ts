// Helpers for the .sma source editor.

/** Character offset of the first character of a 1-based line in `text`. */
export function lineOffset(text: string, line: number): number {
    if (line <= 1) {
        return 0;
    }
    let offset = 0;
    for (let current = 1; current < line; current += 1) {
        const next = text.indexOf('\n', offset);
        if (next === -1) {
            return text.length;
        }
        offset = next + 1;
    }
    return offset;
}
