import React from 'react';
import { Command, SearchResult } from '../types';
import { SearchResultItem } from './SearchResultItem';
import { ArgumentPanel } from './ArgumentPanel';
import { RESULT_LIST_ID } from '../constants';

interface SearchResultListProps {
    results: SearchResult[];
    selectedIndex: number;
    hoveredIndex: number | null;
    argumentMode: { alias: string; command: Command } | null;
    args?: string[];
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
    argumentMode,
    args = [],
    query,
    onResultClick,
    onMouseEnter,
    onMouseLeave,
    onCopy,
    onEdit,
    onDelete,
    onIgnore,
}) => {
    const renderEmptyState = () => {
        if (argumentMode) {
            return <ArgumentPanel command={argumentMode.command} args={args} />;
        }

        if (query) {
            return (
                <div className="flex flex-col items-center justify-center py-12 text-muted h-full select-none">
                    <p>No commands found for "{query}"</p>
                </div>
            );
        }

        return (
            <div className="flex flex-col items-center justify-center py-12 text-muted h-full select-none">
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
                        <span className="text-muted mx-1">/</span>
                        <kbd className="kbd kbd-xs">⇧</kbd>
                        <span className="text-muted">+</span>
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
                        <span className="text-muted">+</span>
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
                <div key="empty" className="h-full">
                    {renderEmptyState()}
                </div>
            ) : (
                // Distinct key forces a remount so orphaned rows can't linger into other modes
                <div key="results" id={RESULT_LIST_ID} role="listbox" aria-label="Search results">
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
