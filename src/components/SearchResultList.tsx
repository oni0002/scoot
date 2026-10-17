import React, { useEffect, useRef } from 'react';
import { Command, SearchResult } from '../types';
import { RowAction } from '../rowActions';
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
    onAction?: (action: RowAction, command: Command, event: React.MouseEvent) => void;
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
    onAction,
    aliasExample,
    onCopyExpanded,
    focusSearch,
}) => {
    const listRef = useRef<HTMLDivElement>(null);

    // Keep the selected row in view as the selection moves
    useEffect(() => {
        listRef.current
            ?.querySelector(`[data-result-index="${selectedIndex}"]`)
            ?.scrollIntoView?.({ behavior: 'smooth', block: 'nearest' });
    }, [selectedIndex]);

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
        <div ref={listRef} className="flex-1 overflow-y-auto min-h-0">
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
                            onAction={onAction}
                            totalCount={results.length}
                            focusSearch={focusSearch}
                        />
                    ))}
                </div>
            )}
        </div>
    );
};
