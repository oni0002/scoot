# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users
Windows users who want a fast, keyboard-centric command launcher. Scoot is published as open source (github.com/oni0002/scoot) with downloadable releases, so people other than its author are expected to install and use it; a first-time user should not get lost. Primary situation: in the middle of other work, summon the launcher, type a few characters, and run something without leaving the keyboard. Beyond "keyboard-centric Windows users" the audience is not narrowed further (not decided).

## Product Purpose
Scoot lets a user reach the things they open all the time from one summoned search box: installed applications, browser bookmarks, links inside their own Markdown notes, and custom commands (URL, file, shell). Success is that the user gets from "I want that" to the thing running in a few keystrokes, and forgets the launcher exists the rest of the time.

## Positioning
Compared with other Windows launchers (for example PowerToys Run, Flow Launcher), Scoot competes on being light and simple, while still searching across more than apps:
- lightweight and low on memory, shipped as a portable single executable with its config next to it;
- deliberately simple, with few features and few settings;
- an alias plus arguments model (`g react` opens a Google search for "react") for running parameterized URLs and commands instantly;
- one fuzzy search across applications, browser bookmarks (Chrome, Brave, Edge), and links extracted from user-chosen Markdown files.

## Operating Context
- Summoned by a global hotkey (default `Alt+Space`) or the tray icon; it hides again when it loses focus, on `Esc`, or after running a command.
- Window: frameless, transparent, always on top, fixed 700x500, hidden from the taskbar.
- Driven entirely from the keyboard: type to fuzzy search, `Tab` / `Shift+Tab` / arrow keys / `Ctrl+N` / `Ctrl+P` to select, `Enter` to run, `Esc` to clear or close, `Ctrl+C` to copy the selected command.
- Search results come from several sources (user commands, bookmarks, applications, Markdown links, built-in Scoot commands) and are labeled by source.
- State lives beside the executable: `config.json`, `commands.json`, and `logs/`. Settings and custom commands are edited in an in-app settings screen and add/edit command form.

## Capabilities and Constraints
- Command categories: `url`, `file`, `command` (shell), and `scoot` (built-in app actions such as settings, reload, quit). Sources: `user`, `bookmark`, `application`, `scoot`.
- Aliases take arguments through placeholders (`{$1}`, `{$2}`, `{$*}`); typing an alias followed by a space enters Argument Mode, where the input becomes arguments instead of search.
- Typing a URL or an absolute path directly offers to open it.
- Settings: theme, global hotkey (captured in-app), max results, fuzzy threshold, reload interval, ignored commands, and per-source options (bookmarks browser, application directories and extensions, Markdown files).
- Themes: `System` (follows the OS), `Light` and `Dark`. Both are Scoot's own: cool neutral greys with a single indigo accent, which marks matched text and the selected alias. Every text and background pair is held to WCAG AA.
- Platform constraints: Windows-first (Tauri v2 desktop app rendered in WebView2); the UI is web technology (React, Tailwind CSS 4, daisyUI 5).
- Command execution is ID-based and URL schemes are allow-listed; these are security invariants, not design choices.
- Domain vocabulary lives in `CONTEXT.md` (Command, Category, Source, Alias, Placeholder, Argument Mode); design decisions are recorded in `docs/adr/`.

## Brand Commitments
The name is "Scoot", and the search placeholder reads "Where do you wanna scoot?". No logo system or voice guide has been defined beyond that; the color identity is the neutral-plus-indigo pair above.

## Evidence on Hand
`README.md` (features, keybindings, configuration, custom commands), `CONTEXT.md`, and `docs/adr/` (alias rename, settings screen design). There are no user testimonials, usage numbers, benchmarks, or press; do not invent any. Memory and size claims ("lightweight") are an intent, not measured figures recorded in the repository.

## Product Principles
- Stay light and simple: a feature earns its place only if it keeps the launcher fast to open and quick to understand.
- Keyboard first: every primary action is reachable without the mouse.
- One search for everything the user already has: apps, bookmarks, notes, and their own commands, not a new place to store things.
- Get out of the way: the launcher appears on demand, does its job, and disappears.
- Be honest about what it can do: no claims or placeholders for capabilities that do not exist.

## Accessibility & Inclusion
No product-specific requirement or standard has been established. Not decided.
