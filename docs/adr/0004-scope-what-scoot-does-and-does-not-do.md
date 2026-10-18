# ADR 0004: What Scoot does and does not do

**Status**: Accepted  
**Date**: 2026-10-18

## Context

Scoot's point of difference is that it is light and simple. Every launcher can grow into a calculator, a clipboard manager and a plugin host; each addition is small, and together they are what makes the others heavy. A feature list that is only ever added to has no way to say no, so the line is written down here, with the proposals that were weighed against it.

## Decision

### The test

A feature belongs in Scoot when it helps with *call it, find a Command, run it*, and it can be built from what is already there. It is out when it needs a new subsystem, a new dependency to carry weight, or makes Scoot do a job another tool does better.

### In scope

- Making search results and their order better (Fuzzy Search, abbreviations, usage history).
- Making a Command easier to reach or to start (what is shown before typing, launching at login).
- Small additions that reuse an existing mechanism.

### Out of scope

- Searching every file on the machine (Everything and the like).
- Other tools' jobs: calculator, unit converter, clipboard history, snippets.
- A plugin system or script runtime.
- Anything that talks to the network on its own: sync, update checks, telemetry.
- A dependency that costs more than the feature gives (a SQLite reader for Firefox bookmarks, an autostart plugin).
- Passing text Scoot did not get from the user's own typing into a shell.

### How the proposals were decided

| Proposal | Decision |
| --- | --- |
| List the most used Commands before typing | **Done.** Ordered by the same `usageBoost` as search, so there is one ranking to explain. Not switchable between "frequent" and "recent": positions stay put for muscle memory, a recent run still counts through the decay, and one or two typed letters already give a history-ranked result. A single on/off setting (`frequentFirst`). |
| Launch at login | **Done as a shortcut in the Startup folder**, made from a switch on the Config Screen. The folder is the source of truth, so nothing goes into `config.json` and the user can see and delete it. `windows` is already in the tree through Tauri; `windows-sys` has no COM interfaces. Moving the exe means switching it off and on again. |
| Web search when nothing matches | **Rejected.** With the default threshold something nearly always matches, so a "no result" fallback would hardly show, and an always-present row would be noise. `g <keyword>` already does this on purpose, and a short word the user wants to look up should show its normal results. |
| Clipboard as an argument (`{$clip}`) | **Deferred.** Clipboard text is not typed by the user, so in a `command` Command it would be an injection path; a Placeholder also forces an Alias and Argument Mode, which a clipboard-only Command does not want; and it has to be added to both `substitute_args` and `substituteArgs`. Revisit only if it can be limited to URL Commands. |
| Run as administrator / reveal in folder by modifier key | **Deferred.** Useful, but not needed yet. |
| Firefox bookmarks | **Deferred.** Needs a SQLite reader for `places.sqlite`; see the dependency rule above. |

## Consequences

- A proposal that fails the test is declined with a pointer to this ADR, not argued again; one that is deferred can come back with the cost it was held for answered.
- New entries go in the table above, with the reason, so the line stays visible.
