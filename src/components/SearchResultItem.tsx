import React from 'react';
import { LuEllipsisVertical, LuCopy } from "react-icons/lu";
import { Command, SearchResult } from '../types';

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
    totalCount: number;
}

// ハイライト処理ユーティリティ (SearchWindowから移動)
const highlightMatches = (text: string, matches: SearchResult['matches']): string => {
    const nameMatch = matches?.find(match => match.key === 'name');
    if (!nameMatch) return text;

    let result = '';
    let lastIndex = 0;

    nameMatch.indices.forEach(([start, end]) => {
        result += text.slice(lastIndex, start);
        result += `<span class="font-semibold text-accent">${text.slice(start, end + 1)}</span>`;
        lastIndex = end + 1;
    });

    return result + text.slice(lastIndex);
};

export const SearchResultItem = React.memo(({
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
    totalCount
}: SearchResultItemProps) => {
    return (
        <div
            data-result-index={index}
            className={`rounded-lg p-2 cursor-pointer flex items-center relative ${isSelected
                ? 'bg-base-content/10'
                : 'bg-base-100 hover:bg-base-200'
                }`}
            onClick={(e) => onClick(index, e)}
            onMouseEnter={() => onMouseEnter(index)}
            onMouseLeave={onMouseLeave}
        >
            <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 text-sm min-w-0">
                    <div
                        className="font-medium truncate"
                        dangerouslySetInnerHTML={{
                            __html: highlightMatches(result.command.name, result.matches)
                        }}
                    />
                    <div className="text-xs opacity-70 truncate flex-1 min-w-0 flex-shrink">
                        {result.command.description || result.command.command}
                    </div>
                </div>
            </div>

            <div className={`flex items-center gap-2 ml-2 flex-shrink-0 ${isHovered ? 'mr-8' : ''}`}>
                {result.command.prompt && (
                    <div className="badge badge-sm badge-primary">
                        {result.command.prompt}
                    </div>
                )}
                <div className="badge badge-sm badge-outline">
                    {result.command.category}
                </div>
            </div>

            {/* 3点ボタンとメニュー */}
            {isHovered && (
                <div className="absolute right-2 top-1/2 transform -translate-y-1/2 z-[1]">
                    <div className={`dropdown ${totalCount > 4 && index >= totalCount - 2 ? 'dropdown-top' : ''} dropdown-end`}>
                        <div tabIndex={0} role="button" className="btn btn-ghost btn-xs btn-square p-1">
                            <LuEllipsisVertical />
                        </div>
                        <ul tabIndex={0} className="dropdown-content menu menu-sm bg-base-100 rounded-lg z-[1] w-40 p-2 shadow">
                            <li>
                                <a onClick={(e) => onCopy?.(result.command, e)}>
                                    <LuCopy className="w-4 h-4" />
                                    Copy
                                </a>
                            </li>
                            {result.command.is_editable && (
                                <>
                                    <li></li>
                                    <li>
                                        <a onClick={(e) => onEdit?.(result.command, e)}>Edit</a>
                                    </li>
                                    <li>
                                        <a className="text-error" onClick={(e) => onDelete?.(result.command, e)}>Delete</a>
                                    </li>
                                </>
                            )}
                        </ul>
                    </div>
                </div>
            )}
        </div>
    );
});

SearchResultItem.displayName = 'SearchResultItem';
