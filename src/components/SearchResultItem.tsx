import React from 'react';
import { LuEllipsisVertical, LuCopy } from 'react-icons/lu';
import { Command, SearchResult } from '../types';
import { resultOptionId } from '../constants';
import { ROW_ACTIONS, RowAction, rowActionsFor } from '../rowActions';

interface SearchResultItemProps {
    result: SearchResult;
    index: number;
    isSelected: boolean;
    isHovered: boolean;
    onClick: (index: number, event: React.MouseEvent) => void;
    onMouseEnter: (index: number) => void;
    onMouseLeave: () => void;
    onCopy?: (command: Command, event: React.MouseEvent) => void;
    onEdit?: (command: Command, event: React.MouseEvent) => void;
    onDelete?: (command: Command, event: React.MouseEvent) => void;
    onIgnore?: (command: Command, event: React.MouseEvent) => void;
    totalCount: number;
}

const highlightMatches = (text: string, matches: SearchResult['matches']): React.ReactNode => {
    const nameMatch = matches?.find((match) => match.key === 'name');
    if (!nameMatch) return text;

    const nodes: React.ReactNode[] = [];
    let lastIndex = 0;

    nameMatch.indices.forEach(([start, end], i) => {
        if (lastIndex < start) nodes.push(text.slice(lastIndex, start));
        nodes.push(
            <span key={i} className="font-semibold text-accent">
                {text.slice(start, end + 1)}
            </span>,
        );
        lastIndex = end + 1;
    });

    if (lastIndex < text.length) nodes.push(text.slice(lastIndex));
    return nodes;
};

export const SearchResultItem = React.memo(
    ({
        result,
        index,
        isSelected,
        isHovered,
        onClick,
        onMouseEnter,
        onMouseLeave,
        onCopy,
        onEdit,
        onDelete,
        onIgnore,
        totalCount,
    }: SearchResultItemProps) => {
        const { command } = result;
        const actions = rowActionsFor(command);
        // The selected row shows its actions too, so keyboard users can see what Ctrl+E/D/I act on
        const showActions = isHovered || isSelected;

        const runAction = (action: RowAction, event: React.MouseEvent) => {
            const handlers: Record<RowAction, typeof onCopy> = {
                copy: onCopy,
                edit: onEdit,
                delete: onDelete,
                ignore: onIgnore,
            };
            handlers[action]?.(command, event);
        };

        // role="none" wrapper: buttons cannot live inside an option, so the actions sit beside it
        return (
            <div
                data-result-index={index}
                role="none"
                className="relative"
                onMouseEnter={() => onMouseEnter(index)}
                onMouseLeave={onMouseLeave}
            >
                <div
                    id={resultOptionId(index)}
                    role="option"
                    aria-selected={isSelected}
                    className={`rounded-lg p-2 cursor-pointer flex items-center ${
                        isSelected
                            ? 'bg-base-content/10 ring-1 ring-inset ring-base-content/30'
                            : 'bg-base-100 hover:bg-base-200'
                    }`}
                    onClick={(e) => onClick(index, e)}
                >
                    <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2 text-sm min-w-0">
                            <div className="font-medium truncate">
                                {highlightMatches(command.name, result.matches)}
                            </div>
                            <div
                                className={`text-xs truncate flex-1 min-w-0 shrink ${isSelected ? '' : 'text-muted'}`}
                            >
                                {command.description || command.command}
                            </div>
                        </div>
                    </div>

                    <div
                        className={`flex items-center gap-2 ml-2 shrink-0 ${showActions ? 'mr-14' : ''}`}
                    >
                        {command.alias && (
                            <div className="badge badge-sm badge-primary">{command.alias}</div>
                        )}
                        <div className="badge badge-sm badge-outline">
                            {command.source || command.category}
                        </div>
                    </div>
                </div>

                {/* Mouse convenience only: keyboard and screen-reader users have Ctrl+C/E/D/I,
                    listed in the hint bar, so these controls stay out of the tab order and tree. */}
                {showActions && (
                    <div
                        aria-hidden="true"
                        className="absolute right-2 top-1/2 -translate-y-1/2 z-[1] flex items-center gap-1"
                    >
                        <button
                            type="button"
                            tabIndex={-1}
                            className="btn btn-ghost btn-xs btn-square p-1"
                            onClick={(e) => onCopy?.(command, e)}
                            title="Copy"
                            aria-label={`Copy ${command.name}`}
                        >
                            <LuCopy />
                        </button>
                        <div
                            className={`dropdown ${totalCount > 4 && index >= totalCount - 2 ? 'dropdown-top' : ''} dropdown-end`}
                        >
                            <div
                                tabIndex={-1}
                                role="button"
                                aria-label={`More actions for ${command.name}`}
                                aria-haspopup="menu"
                                className="btn btn-ghost btn-xs btn-square p-1"
                            >
                                <LuEllipsisVertical />
                            </div>
                            <ul
                                tabIndex={-1}
                                className="dropdown-content menu menu-sm bg-base-100 rounded-lg z-[1] w-48 p-2 shadow-sm"
                            >
                                {actions.map((action) => (
                                    <li key={action}>
                                        <button
                                            type="button"
                                            tabIndex={-1}
                                            className={
                                                ROW_ACTIONS[action].danger ? 'text-error' : ''
                                            }
                                            onClick={(e) => runAction(action, e)}
                                        >
                                            {ROW_ACTIONS[action].label}
                                            <kbd className="kbd kbd-xs ml-auto">
                                                {ROW_ACTIONS[action].shortcut}
                                            </kbd>
                                        </button>
                                    </li>
                                ))}
                            </ul>
                        </div>
                    </div>
                )}
            </div>
        );
    },
);

SearchResultItem.displayName = 'SearchResultItem';
