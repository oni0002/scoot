import { Command } from './types';
import { DIRECT_OPEN_ID } from './constants';
import { hasPlaceholders } from './command';

export type RowAction = 'copy' | 'reveal' | 'edit' | 'duplicate' | 'delete' | 'alias' | 'ignore';

// Single source of truth for what can be done to a result row: the row menu renders from this
// and the keyboard shortcuts resolve through it, so the two cannot drift apart.
export const ROW_ACTIONS: Record<RowAction, { label: string; shortcut: string; danger?: boolean }> =
    {
        copy: { label: 'Copy', shortcut: 'Ctrl+C' },
        reveal: { label: 'Show in folder', shortcut: 'Ctrl+O' },
        edit: { label: 'Edit', shortcut: 'Ctrl+E' },
        duplicate: { label: 'Duplicate', shortcut: 'Ctrl+U' },
        delete: { label: 'Delete', shortcut: 'Ctrl+D', danger: true },
        alias: { label: 'Alias', shortcut: 'Ctrl+L' },
        ignore: { label: 'Ignore', shortcut: 'Ctrl+I' },
    };

// Ctrl+C is handled separately: it must leave a text selection in the input alone.
export const ROW_ACTION_BY_KEY: Record<string, Exclude<RowAction, 'copy'>> = {
    o: 'reveal',
    e: 'edit',
    u: 'duplicate',
    d: 'delete',
    l: 'alias',
    i: 'ignore',
};

/** Actions available for a result. Typed-in URLs and paths can only be copied. */
export function rowActionsFor(command: Command): RowAction[] {
    if (command.id === DIRECT_OPEN_ID) return ['copy'];
    // A custom command carries its alias in the edit form; an external one has no form to edit
    const rest: RowAction[] =
        command.source === 'user' ? ['edit', 'duplicate', 'delete'] : ['alias', 'ignore'];
    return canReveal(command) ? ['copy', 'reveal', ...rest] : ['copy', ...rest];
}

// A file or folder has a place to show; a URL or a shell command does not, and neither does a path
// that still has placeholders to fill in.
function canReveal(command: Command): boolean {
    return command.category === 'file' && !hasPlaceholders(command.command);
}
