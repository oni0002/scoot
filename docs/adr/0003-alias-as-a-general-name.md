# ADR 0003: Alias is a short name for any Command

**Status**: Accepted  
**Date**: 2026-10-18

## Context

An **Alias** was described, and presented in the UI, as the prefix of a Command that takes arguments: the edit form disabled the field unless the command had a Placeholder, and cleared it when the last Placeholder went away. In practice the alias was already a Fuzzy Search key (`src/search.ts`), so it also worked as a short name ("Visual Studio Code" → `vscode`), but nothing said so, and Commands from external Sources (Bookmark, Application, Markdown Link, Scoot Command) could not have one at all.

## Decision

### An alias is a short name for any Command

- Every Command may have an alias. Fuzzy Search matches it like the name.
- A Command with Placeholders **requires** one. Typing `<alias> <args>` then enters Argument Mode.
- On a Command without Placeholders the alias is only a name to search by: `<alias> ` stays a search and never enters Argument Mode.
- The edit form keeps the field enabled and no longer clears it.

### External Commands get aliases from config.json

An external Command is rebuilt on every Reload and its `source` is never persisted, so its alias cannot live in commands.json. It is stored in `config.json` as `aliases`, a map from the Command's `command` string to the alias, and applied when the sources are loaded. This is the same shape as `ignored`.

The map is keyed by the `command` string, **not** by the Command's ID. External IDs are hashes of `source`, `category` and `command` made with std's `DefaultHasher`, whose output Rust does not promise to keep across releases. That is fine for IDs that only live in memory and in the webview, but a persisted hash could silently detach every alias and ignored entry after a toolchain upgrade, and `config.json` travels with the exe. A `command` string is also readable in the Config file. IDs stay for running a Command (`execute_command(id, args)`); `command` strings are for anything saved.

### Setting one: the row menu, not the edit form

An external Command has no edit form, and only its alias can change. Its row menu gets **Alias** (`Ctrl+L`), which opens a small dialog; the value is written to `config.json` at once (like Ignore) through `set_external_alias`. The edit form stays for custom Commands only.

### Uniqueness

An alias is unique across user and external Commands. `CommandRegistry::is_alias_used` looks at both, so the edit form rejects an alias an external Command has, and `set_external_alias` rejects one that is taken or malformed before anything is written. A hand-edited `config.json` that repeats an alias, or reuses one a custom Command has, is repaired on load: the custom Command wins, then the first entry.

## Alternatives considered

- **Two fields** (an alias for arguments, a nickname for search): the same short word would have to be entered twice, and two near-synonyms are harder to explain than one.
- **Keying `aliases` and `ignored` by ID**: see above; the ID is not a stable persistent key.
- **Reusing the edit form for external Commands with every other field disabled**: a form of disabled inputs hides what is editable, and its rules (arguments need an alias, and so on) assume a custom Command.
- **Renaming to `keyword`**: touches every layer and the `commands.json` format for a name that is not clearly better; Raycast and shells use "alias" for the same idea.

## Consequences

- `config.json` gains `aliases`; files without it load as before.
- Aliases of external Commands that disappear (a deleted bookmark, an uninstalled app) stay in `aliases` until removed or until the Command returns, as `ignored` does.
- A new Config field has to be added to the TypeScript `Config` too: the Config Screen saves the whole object, and a missing field would wipe the map.
