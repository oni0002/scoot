import React from 'react';
import { Command } from '../types';
import { ROW_ACTIONS, rowActionsFor } from '../rowActions';

interface SearchHintsProps {
    mode: 'idle' | 'search' | 'argument';
    /** The selected result (search mode only). */
    command?: Command;
}

const hintsFor = ({ mode, command }: SearchHintsProps): Array<[keys: string, label: string]> => {
    if (mode === 'argument') {
        return [
            ['Enter', 'Run'],
            ['Esc', 'Cancel'],
        ];
    }
    if (mode === 'search' && command) {
        return [
            ['Enter', 'Run'],
            ...rowActionsFor(command).map((a): [string, string] => [
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
        <div className="shrink-0 pt-4 flex items-center gap-4 text-xs text-muted select-none">
            {hints.map(([keys, label]) => (
                <span key={label} className="flex items-center gap-1.5">
                    <kbd className="kbd kbd-xs">{keys}</kbd>
                    {label}
                </span>
            ))}
        </div>
    );
};
