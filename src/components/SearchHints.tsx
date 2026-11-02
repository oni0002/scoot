import React from 'react';
import { Command } from '../types';
import { ROW_ACTIONS, rowActionsFor } from '../rowActions';

interface SearchHintsProps {
    mode: 'idle' | 'search' | 'argument';
    /** The selected result (search mode only). */
    command?: Command;
    hasHistory?: boolean;
}

const hintsFor = ({
    mode,
    command,
    hasHistory,
}: SearchHintsProps): Array<[keys: string, label: string]> => {
    if (mode === 'argument') {
        return [
            ['Enter', 'Run'],
            [ROW_ACTIONS.copy.shortcut, ROW_ACTIONS.copy.label],
            ['Esc', 'Cancel'],
        ];
    }
    if (mode === 'search' && command) {
        return [
            ['Enter', 'Run'],
            ...rowActionsFor(command, hasHistory).map((a): [string, string] => [
                ROW_ACTIONS[a].shortcut,
                ROW_ACTIONS[a].label,
            ]),
        ];
    }
    return [];
};

/** One line under the results naming the keys that act on the current selection. */
export const SearchHints: React.FC<SearchHintsProps> = (props) => {
    const hints = hintsFor(props);
    if (hints.length === 0) return null;

    return (
        <div className="shrink-0 pt-4 flex items-center gap-4 text-xs text-muted select-none whitespace-nowrap">
            {hints.map(([keys, label]) => (
                <span key={label} className="flex items-center gap-1.5">
                    <kbd className="kbd kbd-xs">{keys}</kbd>
                    {label}
                </span>
            ))}
        </div>
    );
};
