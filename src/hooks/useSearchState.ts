import { useState, useRef, useCallback, useEffect, useMemo } from 'react';
import { Command, SearchResult, UsageMap } from '../types';
import { createFuse, fuseSearch, frequentCommands, detectDirectOpen } from '../search';
import { hasPlaceholders } from '../command';

const NO_USAGE: UsageMap = {};

export type SearchMode =
    | { mode: 'idle' }
    | { mode: 'argument'; alias: string; command: Command }
    | { mode: 'search'; results: SearchResult[]; selectedIndex: number };

export const useSearchState = (
    commands: Command[],
    fuzzyThreshold: number,
    maxResults: number,
    usage: UsageMap = NO_USAGE,
    showFrequent = false,
) => {
    const [query, setQuery] = useState('');
    const [searchMode, setSearchMode] = useState<SearchMode>({ mode: 'idle' });
    const [idleIndex, setIdleIndex] = useState(0);

    const inputRef = useRef<HTMLInputElement>(null);

    const fuse = useMemo(() => createFuse(commands, fuzzyThreshold), [commands, fuzzyThreshold]);
    // Looked up on every keystroke, so index it once per command list
    const commandByAlias = useMemo(() => {
        const map = new Map<string, Command>();
        for (const cmd of commands) {
            if (cmd.alias && !map.has(cmd.alias)) map.set(cmd.alias, cmd);
        }
        return map;
    }, [commands]);

    // Before anything is typed, the list is derived from the history rather than stored, so it is
    // current each time the window opens and when the history loads late
    const idleResults = useMemo(
        () => (showFrequent ? frequentCommands(commands, usage, maxResults) : []),
        [showFrequent, commands, usage, maxResults],
    );

    const resetState = useCallback(() => {
        setQuery('');
        setSearchMode({ mode: 'idle' });
        setIdleIndex(0);
        inputRef.current?.focus();
    }, []);

    const computeMode = useCallback(
        (newQuery: string): SearchMode => {
            const trimmed = newQuery.trim();

            if (!trimmed) {
                return { mode: 'idle' };
            }

            const parts = trimmed.split(/\s+/);
            const potentialAlias = parts[0];
            const matchingCommand = commandByAlias.get(potentialAlias);

            // An alias on a command without placeholders is only another name to search by
            if (matchingCommand && hasPlaceholders(matchingCommand.command)) {
                const shouldEnter = parts.length > 1 || newQuery.endsWith(' ');
                if (shouldEnter) {
                    return { mode: 'argument', alias: potentialAlias, command: matchingCommand };
                }
                // Still typing the alias keyword — fall through to search
            }

            const searchResults = fuseSearch(fuse, newQuery, commands, maxResults, usage);
            const dynamicItem = detectDirectOpen(newQuery);
            if (dynamicItem) {
                // What was typed is what Enter opens, so it leads the list
                searchResults.unshift({ command: dynamicItem, score: -1, matches: [] });
            }
            return { mode: 'search', results: searchResults, selectedIndex: 0 };
        },
        [commands, commandByAlias, maxResults, fuse, usage],
    );

    const handleQueryChange = useCallback(
        (newQuery: string) => {
            setQuery(newQuery);
            setSearchMode(computeMode(newQuery));
        },
        [computeMode],
    );

    useEffect(() => {
        // eslint-disable-next-line react-hooks/set-state-in-effect -- query-driven mode transition is intentional
        setSearchMode((prev) => {
            // This closure can hold a query that resetState has since cleared (a reload does both
            // at once); idle means nothing is typed, so there is nothing to search again
            if (prev.mode !== 'search') return prev;
            return computeMode(query);
        });
    }, [commands, fuzzyThreshold, maxResults]); // eslint-disable-line react-hooks/exhaustive-deps

    const idleCount = idleResults.length;
    const isIdle = searchMode.mode === 'idle';
    const moveSelection = useCallback(
        (direction: 'up' | 'down') => {
            const step = (index: number, count: number) =>
                direction === 'down' ? (index + 1) % count : (index - 1 + count) % count;
            if (isIdle) {
                if (idleCount > 0) setIdleIndex((index) => step(index, idleCount));
                return;
            }
            setSearchMode((prev) => {
                if (prev.mode !== 'search' || prev.results.length === 0) return prev;
                return { ...prev, selectedIndex: step(prev.selectedIndex, prev.results.length) };
            });
        },
        [isIdle, idleCount],
    );

    const results =
        searchMode.mode === 'search'
            ? searchMode.results
            : searchMode.mode === 'idle'
              ? idleResults
              : [];
    const selectedIndex =
        searchMode.mode === 'search'
            ? searchMode.selectedIndex
            : searchMode.mode === 'idle'
              ? Math.min(idleIndex, Math.max(0, idleCount - 1))
              : 0;
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
