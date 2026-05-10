import { useState, useRef, useCallback, useEffect } from 'react';
import { Command, SearchResult } from '../types';
import { SearchEngine } from '../services/SearchEngine';
import { PromptProcessor } from '../services/PromptProcessor';
import { DirectOpenDetector } from '../services/DirectOpenDetector';

export type SearchMode =
  | { mode: 'idle' }
  | { mode: 'prompt'; prompt: string; command: Command }
  | { mode: 'search'; results: SearchResult[]; selectedIndex: number };

export const useSearchState = (commands: Command[], fuzzyThreshold: number, maxResults: number) => {
    const [query, setQuery] = useState('');
    const [searchMode, setSearchMode] = useState<SearchMode>({ mode: 'idle' });

    const inputRef = useRef<HTMLInputElement>(null);
    const searchEngine = useRef(new SearchEngine(commands, fuzzyThreshold));
    const promptProcessor = useRef(new PromptProcessor(searchEngine.current));

    const resetState = useCallback(() => {
        setQuery('');
        setSearchMode({ mode: 'idle' });
        inputRef.current?.focus();
    }, []);

    const computeMode = useCallback((newQuery: string, currentMode: SearchMode): SearchMode => {
        const trimmed = newQuery.trim();

        if (!trimmed) {
            return { mode: 'idle' };
        }

        const parts = trimmed.split(/\s+/);
        const potentialPrompt = parts[0];
        const matchingCommand = commands.find(cmd => cmd.prompt === potentialPrompt);

        if (matchingCommand) {
            const shouldEnter = parts.length > 1 || newQuery.endsWith(' ');
            if (shouldEnter) {
                return { mode: 'prompt', prompt: potentialPrompt, command: matchingCommand };
            }
            // Still typing the prompt keyword — fall through to search
        } else if (currentMode.mode === 'prompt' && newQuery.startsWith(currentMode.prompt + ' ')) {
            // Typed something after prompt prefix that isn't recognized — stay in prompt
            return currentMode;
        }

        const searchResults = promptProcessor.current.processSearch(newQuery, maxResults);
        const dynamicItem = DirectOpenDetector.detect(newQuery);
        if (dynamicItem) {
            searchResults.push({ command: dynamicItem, score: -1, matches: [] });
        }
        return { mode: 'search', results: searchResults, selectedIndex: 0 };
    }, [commands, maxResults]);

    const handleQueryChange = useCallback((newQuery: string) => {
        setQuery(newQuery);
        setSearchMode(prev => computeMode(newQuery, prev));
    }, [computeMode]);

    useEffect(() => {
        searchEngine.current.updateCommands(commands);
        promptProcessor.current.updateCommands(commands);
        searchEngine.current.updateThreshold(fuzzyThreshold);

        setSearchMode(prev => {
            if (!query || prev.mode === 'prompt') return prev;
            return computeMode(query, prev);
        });
    }, [commands, fuzzyThreshold]); // eslint-disable-line react-hooks/exhaustive-deps

    const moveSelection = useCallback((direction: 'up' | 'down') => {
        setSearchMode(prev => {
            if (prev.mode !== 'search' || prev.results.length === 0) return prev;
            const count = prev.results.length;
            const newIndex = direction === 'down'
                ? (prev.selectedIndex + 1) % count
                : (prev.selectedIndex - 1 + count) % count;
            document.querySelector(`[data-result-index="${newIndex}"]`)
                ?.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
            return { ...prev, selectedIndex: newIndex };
        });
    }, []);

    // Derived values for easy access by consumers
    const results = searchMode.mode === 'search' ? searchMode.results : [];
    const selectedIndex = searchMode.mode === 'search' ? searchMode.selectedIndex : 0;
    const promptMode = searchMode.mode === 'prompt'
        ? { prompt: searchMode.prompt, command: searchMode.command }
        : null;

    return {
        query,
        setQuery,
        searchMode,
        setSearchMode,
        results,
        selectedIndex,
        promptMode,
        inputRef,
        searchEngine,
        promptProcessor,
        resetState,
        handleQueryChange,
        moveSelection,
    };
};
