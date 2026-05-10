import { useCallback } from 'react';
import { Command, SearchResult } from '../types';
import { SearchMode } from './useSearchState';
import { TauriAPI } from '../tauri';

interface UseCommandExecutionProps {
    query: string;
    results: SearchResult[];
    selectedIndex: number;
    promptMode: { prompt: string; command: Command } | null;
    setQuery: (query: string) => void;
    setSearchMode: React.Dispatch<React.SetStateAction<SearchMode>>;
    resetState: () => void;
    executeCommand: (command: Command, args: string[]) => Promise<boolean | null>;
}

export const useCommandExecution = ({
    query,
    results,
    selectedIndex,
    promptMode,
    setQuery,
    setSearchMode,
    resetState,
    executeCommand,
}: UseCommandExecutionProps) => {
    const runSelectedCommand = useCallback(async (targetIndex?: number) => {
        let commandToExecute: Command;
        let argsToPass: string[] = [];

        if (promptMode) {
            commandToExecute = promptMode.command;
            const rest = query.slice(promptMode.prompt.length + 1).trim();
            argsToPass = rest ? rest.split(/\s+/) : [];
        } else {
            const effectiveIndex = targetIndex !== undefined ? targetIndex : selectedIndex;
            if (results.length === 0 || effectiveIndex >= results.length) return;

            const selectedResult = results[effectiveIndex];

            if (selectedResult.command.prompt && !query.startsWith(selectedResult.command.prompt + ' ')) {
                setQuery(selectedResult.command.prompt + ' ');
                setSearchMode({ mode: 'prompt', prompt: selectedResult.command.prompt, command: selectedResult.command });
                return;
            }

            commandToExecute = selectedResult.command;
            const parts = query.trim().split(/\s+/).filter(a => a);
            argsToPass = parts.length <= 1 ? [] : parts;
        }

        const keepWindowOpen = await executeCommand(commandToExecute, argsToPass);
        resetState();

        if (keepWindowOpen === false) {
            TauriAPI.hideWindow();
        }
    }, [results, selectedIndex, query, executeCommand, promptMode, resetState, setSearchMode, setQuery]);

    return { runSelectedCommand };
};
