import React from 'react';
import { Command, SearchResult } from '../types';
import { SearchResultItem } from './SearchResultItem';
import { ArgumentPanel } from './ArgumentPanel';
import { NoResults, QuickGuide } from './EmptyState';
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
    /** An alias that takes arguments, for the first-run guide to use as its example. */
    aliasExample?: string;
    /** Copies the expanded command while in argument mode. */
    onCopyExpanded?: () => void;
    focusSearch?: () => void;
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
    aliasExample,
    onCopyExpanded,
    focusSearch,
}) => {
    const renderEmptyState = () => {
        if (argumentMode) {
            return (
                <ArgumentPanel command={argumentMode.command} args={args} onCopy={onCopyExpanded} />
            );
        }

        if (query) return <NoResults query={query} />;

        return <QuickGuide aliasExample={aliasExample} />;
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
                            showActions={
                                hoveredIndex !== null
                                    ? hoveredIndex === index
                                    : index === selectedIndex
                            }
                            onClick={onResultClick}
                            onMouseEnter={onMouseEnter}
                            onMouseLeave={onMouseLeave}
                            onCopy={onCopy}
                            onEdit={onEdit}
                            onDelete={onDelete}
                            onIgnore={onIgnore}
                            totalCount={results.length}
                            focusSearch={focusSearch}
                        />
                    ))}
                </div>
            )}
        </div>
    );
};
