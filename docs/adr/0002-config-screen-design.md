# ADR 0002: Config Screen design decisions

**Status**: Accepted  
**Date**: 2026-06-04

## Context

Config was previously only editable by opening `config.json` directly in a text editor (via `openConfigJson` Scoot Command). A GUI Config Screen was needed to make Config accessible to non-technical users and to provide a better editing experience for structured fields like arrays and the Hotkey.

## Decisions

### Immediate save (no Save button)

Config fields save on change, validated before writing. This matches the UX pattern of modern desktop settings screens (VS Code, macOS System Settings) and aligns with Scoot's "simple and clean" principle. A cancel/discard flow was considered unnecessary given that the save target is a local JSON file with no latency.

**Amended**: the write happens when the screen closes (Esc or Back), not per field, so one validation covers every edit. A failed write keeps the screen open and shows the reason, with a "Discard and close" escape; a successful one shows a "Settings saved" toast.

**Alternative considered**: Save button with cancel. Rejected because it adds UI chrome for no practical benefit in a local-file setting.

### `ignored` list: remove only

The Config Screen shows the `ignored` list with delete buttons (un-ignore). Adding to `ignored` is only possible via the Ignore button on search results, where the exact `command` string is already known. Allowing free-text addition in Config Screen would require users to know the internal `command` field value, which is poor UX.

**Alternative considered**: Add + remove in Config Screen. Rejected due to UX friction of hand-typing internal command strings.

### Hotkey: key-capture UI

The Hotkey field uses a key-capture input: focus the field, press the desired key combination, the UI records it as `Modifier+Key` format (e.g. `Alt+Space`). Free-text input was rejected because intermediate keystrokes during typing produce incomplete values that cannot be meaningfully validated or saved.

### Theme: live preview

Selecting a theme (System / Light / Dark) immediately applies `data-theme` to the Config Screen itself, giving instant visual feedback. Since save is immediate, the selected theme is the confirmed theme.

## Consequences

- `currentView` in `App.tsx` gains a third state: `'config'`.
- Config Screen is opened via Scoot Command (`OpenSettings`) and the 3-dot menu in SearchWindow; `openConfigJson` remains as a separate menu item for advanced users.
- The `ignored` list has a deliberate asymmetry: additions happen in the search flow, removals happen in Config Screen.
