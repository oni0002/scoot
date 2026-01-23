import React, { useState } from 'react';
import { LuCopy, LuCheck } from 'react-icons/lu';
import { Command, SearchResult } from '../types';
import { SearchResultItem } from './SearchResultItem';

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
    onDelete
}) => {
    const [copied, setCopied] = useState(false);

    const handleCopyPreview = (text: string) => {
        navigator.clipboard.writeText(text);
        setCopied(true);
        setTimeout(() => setCopied(false), 3000);
    };
    // プレビュー生成ロジック
    const getPreviewCommand = (command: Command, args: string[]) => {
        let cmd = command.command;
        if (!cmd) return '';

        if (args.length === 0) return cmd;

        // {$*} replacement
        if (cmd.includes('{$*}')) {
            return cmd.replace('{$*}', args.join(' '));
        }

        // {$1}, {$2}... replacement
        let hasNumbered = false;
        args.forEach((arg, index) => {
            const placeholder = `{$${index + 1}}`;
            if (cmd.includes(placeholder)) {
                cmd = cmd.replace(placeholder, arg);
                hasNumbered = true;
            }
        });
        return cmd;
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

        return (
            <div className="text-center py-8 opacity-70">
                {query ? 'No commands found' : 'Start typing to search...'}
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
                            totalCount={results.length}
                        />
                    ))}
                </div>
            )}
        </div>
    );
};
