# Reddit post — GoldSource Addons announcement

## Title (pick one)

- New official plugin: GoldSource Addons — manage Metamod & AMX Mod X from the panel
- Introducing GoldSource Addons: Metamod / AMX Mod X plugin management for GoldSrc servers

## Body

We've just released a new official GameAP plugin: **GoldSource Addons**. It brings
Metamod and AMX Mod X plugin management straight into the panel for GoldSource
servers — Half-Life 1, Counter-Strike 1.6, and other GoldSrc mods.

It adds a **Plugins** tab to the server page, so no more SSH-ing in to hand-edit
`plugins.ini`.

**What it can do:**

- **Status at a glance** — Metamod / AMX Mod X cards (installed · inactive · not
  installed), versions pulled live from the server console over RCON, plugin counts.
- **Full plugin list** for both AMXX and Metamod, grouped by the sections of
  `plugins.ini`, with a live per-plugin status: Running / Awaiting restart /
  Error (bad load) / File missing / Disabled — plus version and author from the console.
- **Enable / disable** with a switch. It comments/uncomments the line in
  `plugins.ini` and leaves the rest of the file byte-for-byte intact (CRLF, BOM,
  CP1251 comments and all).
- **Install from file** — drag & drop `.amxx` (AMX Mod X) or `.so`/`.dll` (Metamod);
  re-uploading overwrites/updates in place. `.sma` sources are rejected with a hint
  to compile them first.
- **Edit plugin configs** (`configs/<name>.cfg`) in a modal, right in the browser.
- **Per-plugin extras** — toggle the AMX Mod X `debug` flag, edit inline comments,
  bulk enable/disable/delete, search & filter.
- **Restart handling** — a "restart required" banner with a one-click restart
  (changes take effect after a restart / map change).

**Good to know:**

- It manages *plugins*; installing Metamod / AMX Mod X themselves is out of scope.
- The tab shows up only on **GoldSource** servers, and only for admins.
- Live versions/statuses need the server running with an RCON password set —
  otherwise statuses fall back to what's written in `plugins.ini`.

This is an early release (v0.1.0) — feedback, bug reports and feature requests are
very welcome.
