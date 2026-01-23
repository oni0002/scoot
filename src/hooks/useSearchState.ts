import { useState, useRef, useCallback, useEffect } from 'react';
import { Command, SearchResult } from '../types';
import { SearchEngine } from '../services/SearchEngine';
import { PromptProcessor } from '../services/PromptProcessor';


export const useSearchState = (commands: Command[], fuzzyThreshold: number) => {
    const [query, setQuery] = useState('');
    const [results, setResults] = useState<SearchResult[]>([]);
    const [selectedIndex, setSelectedIndex] = useState(0);
    const [promptMode, setPromptMode] = useState<{ prompt: string; command: Command } | null>(null);

    const inputRef = useRef<HTMLInputElement>(null);
    const searchEngine = useRef(new SearchEngine(commands, fuzzyThreshold));
    const promptProcessor = useRef(new PromptProcessor(searchEngine.current));

    // 状態リセット
    const resetState = useCallback(() => {
        setQuery('');
        setResults([]);
        setSelectedIndex(0);
        setPromptMode(null);
        inputRef.current?.focus();
    }, []);

    // 検索とプロンプトモード判定を一括処理
    const updateSearchState = useCallback((newQuery: string, currentPromptMode: { prompt: string; command: Command } | null) => {
        const trimmed = newQuery.trim();

        if (!trimmed) {
            if (currentPromptMode) {
                setPromptMode(null);
            }
            setResults([]);
            setSelectedIndex(0);
            return;
        }

        const parts = trimmed.split(/\s+/);

        // プロンプトモード判定
        let nextPromptMode = currentPromptMode;
        let shouldSearch = true;

        if (parts.length > 0) {
            const potentialPrompt = parts[0];
            const matchingCommand = commands.find(cmd => cmd.prompt === potentialPrompt);

            if (matchingCommand) {
                const shouldEnter = parts.length > 1 || newQuery.endsWith(' ');
                if (shouldEnter) {
                    if (!currentPromptMode || currentPromptMode.prompt !== potentialPrompt) {
                        nextPromptMode = { prompt: potentialPrompt, command: matchingCommand };
                        setPromptMode(nextPromptMode);
                    }
                    shouldSearch = false;
                } else if (currentPromptMode) {
                    nextPromptMode = null;
                    setPromptMode(null);
                }
            } else if (currentPromptMode && !newQuery.startsWith(currentPromptMode.prompt + ' ')) {
                nextPromptMode = null;
                setPromptMode(null);
            }
        } else {
            if (currentPromptMode) {
                nextPromptMode = null;
                setPromptMode(null);
            }
            shouldSearch = false;
            setResults([]);
            setSelectedIndex(0);
        }

        // 検索実行
        if (shouldSearch) {
            // プロンプトモード中は検索結果をクリア
            if (nextPromptMode) {
                setResults([]);
                setSelectedIndex(0);
            } else {
                const searchResults = promptProcessor.current.processSearch(newQuery, 30);
                setResults(searchResults);
                setSelectedIndex(0);
            }
        } else if (nextPromptMode) {
            // プロンプトモード確定時など、検索を行わない場合でも結果をクリアする
            setResults([]);
            setSelectedIndex(0);
        }
    }, [commands]);

    // エンジン更新
    useEffect(() => {
        searchEngine.current.updateCommands(commands);
        promptProcessor.current.updateCommands(commands);
        searchEngine.current.updateThreshold(fuzzyThreshold);

        // コマンドリスト更新時に検索結果も更新（クエリがある場合のみ）
        if (query && !promptMode) {
            updateSearchState(query, promptMode);
        }
    }, [commands, fuzzyThreshold]); // Removed query/promptMode from deps to avoid loop, logic handled inside update

    // クエリ変更処理
    const handleQueryChange = useCallback((newQuery: string) => {
        setQuery(newQuery);
        updateSearchState(newQuery, promptMode);
    }, [updateSearchState, promptMode]);

    // 選択項目の移動
    const moveSelection = useCallback((direction: 'up' | 'down') => {
        if (results.length === 0) return;

        const newIndex = direction === 'down'
            ? (selectedIndex + 1) % results.length
            : (selectedIndex - 1 + results.length) % results.length;

        setSelectedIndex(newIndex);

        // スクロール調整
        document.querySelector(`[data-result-index="${newIndex}"]`)
            ?.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
    }, [results.length, selectedIndex]);

    return {
        query,
        setQuery,
        results,
        setResults,
        selectedIndex,
        setSelectedIndex,
        promptMode,
        setPromptMode,
        inputRef,
        searchEngine,
        promptProcessor,
        resetState,
        handleQueryChange,
        moveSelection
    };
};
