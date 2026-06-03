import React from 'react';
import { LuEllipsisVertical, LuCopy } from 'react-icons/lu';
import { Command, SearchResult } from '../types';
import { DIRECT_OPEN_ID } from '../constants';

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
        return (
            <div
                data-result-index={index}
                className={`rounded-lg p-2 cursor-pointer flex items-center relative ${
                    isSelected ? 'bg-base-content/10' : 'bg-base-100 hover:bg-base-200'
                }`}
                onClick={(e) => onClick(index, e)}
                onMouseEnter={() => onMouseEnter(index)}
                onMouseLeave={onMouseLeave}
            >
                <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 text-sm min-w-0">
                        <div className="font-medium truncate">
                            {highlightMatches(result.command.name, result.matches)}
                        </div>
                        <div className="text-xs opacity-70 truncate flex-1 min-w-0 flex-shrink">
                            {result.command.description || result.command.command}
                        </div>
                    </div>
                </div>

                <div
                    className={`flex items-center gap-2 ml-2 flex-shrink-0 ${isHovered ? 'mr-14' : ''}`}
                >
                    {result.command.prompt && (
                        <div className="badge badge-sm badge-primary">{result.command.prompt}</div>
                    )}
                    <div className="badge badge-sm badge-outline">
                        {result.command.source || result.command.category}
                    </div>
                </div>

                {/* Quick Actions & Dropdown menu */}
                {isHovered && (
                    <div className="absolute right-2 top-1/2 transform -translate-y-1/2 z-[1] flex items-center gap-1">
                        <button
                            className="btn btn-ghost btn-xs btn-square p-1"
                            onClick={(e) => onCopy?.(result.command, e)}
                            title="Copy"
                        >
                            <LuCopy />
                        </button>
                        <div
                            className={`dropdown ${totalCount > 4 && index >= totalCount - 2 ? 'dropdown-top' : ''} dropdown-end`}
                        >
                            <div
                                tabIndex={0}
                                role="button"
                                className="btn btn-ghost btn-xs btn-square p-1"
                            >
                                <LuEllipsisVertical />
                            </div>
                            <ul
                                tabIndex={0}
                                className="dropdown-content menu menu-sm bg-base-100 rounded-lg z-[1] w-40 p-2 shadow"
                            >
                                <li>
                                    <a onClick={(e) => onCopy?.(result.command, e)}>Copy</a>
                                </li>
                                {result.command.source === 'user' &&
                                    result.command.id !== DIRECT_OPEN_ID && (
                                        <>
                                            <li>
                                                <a onClick={(e) => onEdit?.(result.command, e)}>
                                                    Edit
                                                </a>
                                            </li>
                                            <li>
                                                <a
                                                    className="text-error"
                                                    onClick={(e) => onDelete?.(result.command, e)}
                                                >
                                                    Delete
                                                </a>
                                            </li>
                                        </>
                                    )}
                                {result.command.source !== 'user' && (
                                    <li>
                                        <a onClick={(e) => onIgnore?.(result.command, e)}>Ignore</a>
                                    </li>
                                )}
                            </ul>
                        </div>
                    </div>
                )}
            </div>
        );
    },
);

SearchResultItem.displayName = 'SearchResultItem';
