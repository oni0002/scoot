# ADR 0005: Settings replaces Config, and settings.json replaces config.json

**Status**: Accepted  
**Date**: 2026-10-19

## Context

The UI has always said "Settings" (the screen heading, the menu, the notices, the README), while the glossary, the code and the file said "Config" (`Config`, `ConfigScreen`, `ConfigContext`, `save_config`, `config.json`). Two words for one thing; the glossary had to carve out an exception for screen text.

The file lives next to the exe and users edit it by hand, so renaming it must not lose anyone's settings, including someone who skips versions or goes back to an older one.

## Decision

### The file is `settings.json`

- **Both exist**: `settings.json` wins. `config.json` is left alone, and the load logs that it is ignored.
- **Only `config.json` exists** (first start after the update): it is renamed to `settings.json`, then read. If the rename fails (read-only folder, a lock), `config.json` is read as it is, with a warning in the log; the next load tries the rename again.
- **Neither exists**: `settings.json` is created with the defaults.
- **Writes** always go to `settings.json`. Nothing ever writes `config.json`.
- The file watcher watches both names, so an edit is picked up whichever file is in use.
- "Open settings.json" (menu, tray, Scoot Command) opens the file in use.

Going back to an older version generates a fresh `config.json` with the defaults there; the user copies values over from `settings.json` by hand. That is the cost of not keeping two files in sync, and it is rare.

### Compatibility ends one minor series later

The fallback to `config.json` ships in 0.3.0, stays through 0.3.x, and is removed in 0.4.0. It is not removed in the very next release because the app is portable: someone who jumps from 0.2.x straight past the series that renames the file would fall back to the defaults without a word. Passing through any 0.3.x release migrates the file on its first load.

### Names move to "Settings" step by step

Not in one change: the file layer first (this ADR's decisions above), then, in separate commits, the backend (module, types, commands, events), the frontend (types, context, screen, `TauriAPI`), and finally CONTEXT.md and the README. Each step keeps the tests green. The glossary term **Config** becomes **Settings** at the last step, with **Config Screen** → **Settings Screen**; until then CONTEXT.md keeps the old words so the code and the glossary agree at every commit.

## Consequences

- One word everywhere once the steps are done, and no exception in the glossary for screen text.
- For one minor series the load code has a fallback that can be deleted whole in 0.4.0.
- The ADRs written before this one (0002, 0003) still say `config.json`; they are records and are not rewritten.
