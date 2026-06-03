import { useState, useRef, useCallback, useEffect, useMemo } from 'react';
import { Command, SearchResult } from '../types';
import { createFuse, fuseSearch, detectDirectOpen } from '../search';

export type SearchMode =
    | { mode: 'idle' }
    | { mode: 'argument'; alias: string; command: Command }
    | { mode: 'search'; results: SearchResult[]; selectedIndex: number };

export const useSearchState = (commands: Command[], fuzzyThreshold: number, maxResults: number) => {
    const [query, setQuery] = useState('');
    const [searchMode, setSearchMode] = useState<SearchMode>({ mode: 'idle' });

    const inputRef = useRef<HTMLInputElement>(null);

    const fuse = useMemo(() => createFuse(commands, fuzzyThreshold), [commands, fuzzyThreshold]);

    const resetState = useCallback(() => {
        setQuery('');
        setSearchMode({ mode: 'idle' });
        inputRef.current?.focus();
    }, []);

    const computeMode = useCallback(
        (newQuery: string, currentMode: SearchMode): SearchMode => {
            const trimmed = newQuery.trim();

            if (!trimmed) {
                return { mode: 'idle' };
            }

            const parts = trimmed.split(/\s+/);
            const potentialAlias = parts[0];
            const matchingCommand = commands.find((cmd) => cmd.alias === potentialAlias);

            if (matchingCommand) {
                const shouldEnter = parts.length > 1 || newQuery.endsWith(' ');
                if (shouldEnter) {
                    return { mode: 'argument', alias: potentialAlias, command: matchingCommand };
                }
                // Still typing the alias keyword — fall through to search
            } else if (
                currentMode.mode === 'argument' &&
                newQuery.startsWith(currentMode.alias + ' ')
            ) {
                // Typed something after alias prefix that isn't recognized — stay in argument mode
                return currentMode;
            }

            const searchResults = fuseSearch(fuse, newQuery, commands, maxResults);
            const dynamicItem = detectDirectOpen(newQuery);
            if (dynamicItem) {
                searchResults.push({ command: dynamicItem, score: -1, matches: [] });
            }
            return { mode: 'search', results: searchResults, selectedIndex: 0 };
        },
        [commands, maxResults, fuse],
    );

    const handleQueryChange = useCallback(
        (newQuery: string) => {
            setQuery(newQuery);
            setSearchMode((prev) => computeMode(newQuery, prev));
        },
        [computeMode],
    );

    useEffect(() => {
        // eslint-disable-next-line react-hooks/set-state-in-effect -- query-driven mode transition is intentional
        setSearchMode((prev) => {
            if (!query || prev.mode === 'argument') return prev;
            return computeMode(query, prev);
        });
    }, [commands, fuzzyThreshold]); // eslint-disable-line react-hooks/exhaustive-deps

    const moveSelection = useCallback((direction: 'up' | 'down') => {
        setSearchMode((prev) => {
            if (prev.mode !== 'search' || prev.results.length === 0) return prev;
            const count = prev.results.length;
            const newIndex =
                direction === 'down'
                    ? (prev.selectedIndex + 1) % count
                    : (prev.selectedIndex - 1 + count) % count;
            document
                .querySelector(`[data-result-index="${newIndex}"]`)
                ?.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
            return { ...prev, selectedIndex: newIndex };
        });
    }, []);

    const results = searchMode.mode === 'search' ? searchMode.results : [];
    const selectedIndex = searchMode.mode === 'search' ? searchMode.selectedIndex : 0;
    const argumentMode =
        searchMode.mode === 'argument'
            ? { alias: searchMode.alias, command: searchMode.command }
            : null;

    return {
        query,
        setQuery,
        searchMode,
        setSearchMode,
        results,
        selectedIndex,
        argumentMode,
        inputRef,
        resetState,
        handleQueryChange,
        moveSelection,
    };
};
