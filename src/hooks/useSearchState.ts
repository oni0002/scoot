import { useState, useRef, useCallback, useEffect, useMemo } from 'react';
import { Command, SearchResult, UsageMap } from '../types';
import {
    createIndex,
    searchIndex,
    SearchIndex,
    frequentCommands,
    detectDirectOpen,
} from '../search';
import { hasPlaceholders } from '../command';

const NO_USAGE: UsageMap = {};

export type SearchMode =
    | { mode: 'idle' }
    | { mode: 'argument'; alias: string; command: Command }
    | { mode: 'search'; results: SearchResult[]; selectedIndex: number; scoped?: boolean };

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

    // alias_only の登録フォルダの項目は、そのフォルダの Alias を打ったときだけ探せる
    const searchable = useMemo(() => commands.filter((c) => !c.scopedOnly), [commands]);
    const index = useMemo(
        () => createIndex(searchable, fuzzyThreshold),
        [searchable, fuzzyThreshold],
    );
    // Local Path の Alias ごとの索引。打鍵のたびに作らないよう、コマンド一覧か閾値が変わったときに作る
    const scopeIndexes = useMemo(() => {
        const byAlias = new Map<string, Command[]>();
        for (const cmd of commands) {
            for (const alias of cmd.scopes ?? []) {
                byAlias.set(alias, [...(byAlias.get(alias) ?? []), cmd]);
            }
        }
        return new Map<string, SearchIndex>(
            [...byAlias].map(([alias, scoped]) => [alias, createIndex(scoped, fuzzyThreshold)]),
        );
    }, [commands, fuzzyThreshold]);
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
        () => (showFrequent ? frequentCommands(searchable, usage, maxResults) : []),
        [showFrequent, searchable, usage, maxResults],
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

            // Local Path の Alias は、そのフォルダの下だけを探す。引数を取る Command の Alias が先
            const scopeIndex = scopeIndexes.get(potentialAlias);
            if (scopeIndex && (parts.length > 1 || newQuery.endsWith(' '))) {
                const keyword = trimmed.slice(potentialAlias.length).trim();
                const results = searchIndex(scopeIndex, keyword, maxResults, usage);
                return { mode: 'search', results, selectedIndex: 0, scoped: true };
            }

            const searchResults = searchIndex(index, newQuery, maxResults, usage);
            const dynamicItem = detectDirectOpen(newQuery);
            if (dynamicItem) {
                // What was typed is what Enter opens, so it leads the list
                searchResults.unshift({ command: dynamicItem, score: -1, matches: [] });
            }
            return { mode: 'search', results: searchResults, selectedIndex: 0 };
        },
        [commandByAlias, maxResults, index, usage, scopeIndexes],
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
        scoped: searchMode.mode === 'search' && Boolean(searchMode.scoped),
        inputRef,
        resetState,
        handleQueryChange,
        moveSelection,
    };
};
