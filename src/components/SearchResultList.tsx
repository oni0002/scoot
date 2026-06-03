import React, { useState } from 'react';
import { LuCopy, LuCheck } from 'react-icons/lu';
import { Command, SearchResult } from '../types';
import { SearchResultItem } from './SearchResultItem';
import { substituteArgs } from '../command';

interface SearchResultListProps {
    results: SearchResult[];
    selectedIndex: number;
    hoveredIndex: number | null;
    promptMode: { prompt: string; command: Command } | null;
    promptArgs?: string[];
    query: string;
    onResultClick: (index: number, event?: React.MouseEvent) => void;
    onMouseEnter: (index: number) => void;
    onMouseLeave: () => void;
    onCopy?: (command: Command, event: React.MouseEvent) => void;
    onEdit?: (command: Command, event: React.MouseEvent) => void;
    onDelete?: (command: Command, event: React.MouseEvent) => void;
    onIgnore?: (command: Command, event: React.MouseEvent) => void;
}

export const SearchResultList: React.FC<SearchResultListProps> = ({
    results,
    selectedIndex,
    hoveredIndex,
    promptMode,
    promptArgs = [],
    query,
    onResultClick,
    onMouseEnter,
    onMouseLeave,
    onCopy,
    onEdit,
    onDelete,
    onIgnore,
}) => {
    const [copied, setCopied] = useState(false);

    const handleCopyPreview = (text: string) => {
        navigator.clipboard.writeText(text);
        setCopied(true);
        setTimeout(() => setCopied(false), 3000);
    };
    const getPreviewCommand = (command: Command, args: string[]) => {
        if (!command.command) return '';
        return substituteArgs(command.command, args);
    };

    const renderEmptyState = () => {
        if (promptMode) {
            const preview = getPreviewCommand(promptMode.command, promptArgs);
            return (
                <div className="text-center py-8">
                    <div className="text-lg mb-1">{promptMode.command.name}</div>
                    <div className="text-xs opacity-70 mb-4">
                        {promptMode.command.description || 'Enter arguments'}
                    </div>
                    {preview && (
                        <div className="flex items-center p-2 mx-8 gap-2 text-center bg-base-200 rounded-lg">
                            <div className="flex-1 min-w-0 text-xs font-mono break-all">
                                {preview}
                            </div>
                            <button
                                onClick={() => handleCopyPreview(preview)}
                                className="flex-none btn btn-square btn-xs btn-ghost text-base-content/50 hover:text-base-content transition-colors"
                                title="Copy"
                            >
                                {copied ? <LuCheck /> : <LuCopy />}
                            </button>
                        </div>
                    )}
                </div>
            );
        }

        if (query) {
            return (
                <div className="flex flex-col items-center justify-center py-12 text-base-content/50 h-full select-none">
                    <p>No commands found for "{query}"</p>
                </div>
            );
        }

        return (
            <div className="flex flex-col items-center justify-center py-12 text-base-content/50 h-full select-none">
                <div className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-sm">
                    <div className="text-right">
                        <span>Search</span>
                    </div>
                    <div className="flex items-center gap-1 font-mono text-xs">
                        <span>Type any keyword...</span>
                    </div>

                    <div className="text-right">
                        <span>Navigate</span>
                    </div>
                    <div className="flex items-center gap-1">
                        <kbd className="kbd kbd-xs">Tab</kbd>
                        <span className="opacity-50 mx-1">/</span>
                        <kbd className="kbd kbd-xs">⇧</kbd>
                        <span className="opacity-50">+</span>
                        <kbd className="kbd kbd-xs">Tab</kbd>
                    </div>

                    <div className="text-right">
                        <span>Execute</span>
                    </div>
                    <div className="flex items-center gap-1">
                        <kbd className="kbd kbd-xs">↵ Enter</kbd>
                    </div>

                    <div className="text-right">
                        <span>Toggle Window</span>
                    </div>
                    <div className="flex items-center gap-1">
                        <kbd className="kbd kbd-xs">Alt</kbd>
                        <span className="opacity-50">+</span>
                        <kbd className="kbd kbd-xs">Space</kbd>
                    </div>

                    <div className="text-right">
                        <span>Close</span>
                    </div>
                    <div className="flex items-center gap-1">
                        <kbd className="kbd kbd-xs">Esc</kbd>
                    </div>
                </div>
            </div>
        );
    };

    return (
        <div className="flex-1 overflow-y-auto min-h-0">
            {results.length === 0 ? (
                renderEmptyState()
            ) : (
                <div>
                    {results.map((result, index) => (
                        <SearchResultItem
                            key={result.command.id}
                            index={index}
                            result={result}
                            isSelected={index === selectedIndex}
                            isHovered={hoveredIndex === index}
                            onClick={onResultClick}
                            onMouseEnter={onMouseEnter}
                            onMouseLeave={onMouseLeave}
                            onCopy={onCopy}
                            onEdit={onEdit}
                            onDelete={onDelete}
                            onIgnore={onIgnore}
                            totalCount={results.length}
                        />
                    ))}
                </div>
            )}
        </div>
    );
};
