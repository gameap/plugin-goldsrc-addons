// File-name helpers for plugin uploads.

/**
 * Directory name for a Metamod plugin binary:
 * reunion_mm_i386.so → reunion, podbot_mm.dll → podbot,
 * VoiceTranscoder.so → VoiceTranscoder.
 */
export function metamodDirName(fileName: string): string {
    let stem = fileName.replace(/\.(so|dll)$/i, '');
    stem = stem.replace(/_(i[36]86|amd64|x64|x86)$/i, '');
    stem = stem.replace(/_mm$/i, '');
    return stem;
}

export function fileExtension(fileName: string): string {
    const idx = fileName.lastIndexOf('.');
    return idx > 0 ? fileName.slice(idx + 1).toLowerCase() : '';
}

export function fileStem(fileName: string): string {
    const idx = fileName.lastIndexOf('.');
    return idx > 0 ? fileName.slice(0, idx) : fileName;
}

/** Human-ish plugin name from a file name: high_ping_kicker.amxx → High Ping Kicker. */
export function prettyName(fileName: string): string {
    return fileStem(fileName)
        .replace(/[_-]+/g, ' ')
        .replace(/\b\w/g, (ch) => ch.toUpperCase());
}
