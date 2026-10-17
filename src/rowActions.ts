import { Command } from './types';
import { DIRECT_OPEN_ID } from './constants';

export type RowAction = 'copy' | 'edit' | 'delete' | 'ignore';

// Single source of truth for what can be done to a result row: the row menu renders from this
// and the keyboard shortcuts resolve through it, so the two cannot drift apart.
export const ROW_ACTIONS: Record<RowAction, { label: string; shortcut: string; danger?: boolean }> =
    {
        copy: { label: 'Copy', shortcut: 'Ctrl+C' },
        edit: { label: 'Edit', shortcut: 'Ctrl+E' },
        delete: { label: 'Delete', shortcut: 'Ctrl+D', danger: true },
        ignore: { label: 'Ignore', shortcut: 'Ctrl+I' },
    };

// Ctrl+C is handled separately: it must leave a text selection in the input alone.
export const ROW_ACTION_BY_KEY: Record<string, Exclude<RowAction, 'copy'>> = {
    e: 'edit',
    d: 'delete',
    i: 'ignore',
};

/** Actions available for a result. Typed-in URLs and paths can only be copied. */
export function rowActionsFor(command: Command): RowAction[] {
    if (command.id === DIRECT_OPEN_ID) return ['copy'];
    return command.source === 'user' ? ['copy', 'edit', 'delete'] : ['copy', 'ignore'];
}
