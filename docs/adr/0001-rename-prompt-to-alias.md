# ADR 0001: Rename `prompt` to `alias` across the codebase

**Status**: Accepted  
**Date**: 2026-06-04

## Context

The field that stores a Command's short invocation keyword was named `prompt` in all code and in the `commands.json` persistence format. The domain language (CONTEXT.md) had already established **Alias** as the correct term for this concept, creating a mismatch between the code and the ubiquitous language.

## Decision

Rename `prompt` → `alias` everywhere: Rust struct fields, internal indices, IPC function names, TypeScript types, component props, state variables, and UI labels.

The UI state previously called `promptMode` is renamed to `argumentMode` to align with the domain term **Argument Mode** (the state where the user is supplying arguments after typing an alias).

To preserve backward compatibility with existing `commands.json` files that contain `"prompt": "..."` entries, the Rust struct retains a serde alias:

```rust
#[serde(default, skip_serializing_if = "Option::is_none", alias = "prompt")]
pub alias: Option<String>,
```

This means old files are read correctly; new saves write `"alias"`.

## Alternatives considered

- **Keep `prompt` in code**: Perpetuates the mismatch and confuses contributors who read CONTEXT.md.
- **Migrate `commands.json` in place**: Would require a one-time migration step; the serde alias achieves the same result transparently without a migration script.

## Consequences

- Code and domain language are now consistent.
- The `#[serde(alias = "prompt")]` annotation will remain permanently to support any existing `commands.json` files in the wild.
