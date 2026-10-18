import { useCallback } from 'react';
import { Command, SearchResult } from '../types';
import { SearchMode } from './useSearchState';
import { TauriAPI } from '../tauri';
import { hasPlaceholders, missingSlots, parseArgs } from '../command';

interface UseCommandExecutionProps {
    query: string;
    results: SearchResult[];
    selectedIndex: number;
    argumentMode: { alias: string; command: Command } | null;
    setQuery: (query: string) => void;
    setSearchMode: React.Dispatch<React.SetStateAction<SearchMode>>;
    resetState: () => void;
    executeCommand: (command: Command, args: string[]) => Promise<boolean | null>;
}

export const useCommandExecution = ({
    query,
    results,
    selectedIndex,
    argumentMode,
    setQuery,
    setSearchMode,
    resetState,
    executeCommand,
}: UseCommandExecutionProps) => {
    const runSelectedCommand = useCallback(
        async (targetIndex?: number) => {
            let commandToExecute: Command;
            let argsToPass: string[];

            if (argumentMode) {
                commandToExecute = argumentMode.command;
                argsToPass = parseArgs(query, argumentMode.alias);
                // The panel shows what it is still waiting for; the backend would refuse too
                if (missingSlots(commandToExecute.command, argsToPass).length > 0) return;
            } else {
                const effectiveIndex = targetIndex !== undefined ? targetIndex : selectedIndex;
                if (results.length === 0 || effectiveIndex >= results.length) return;

                const selectedResult = results[effectiveIndex];

                if (
                    selectedResult.command.alias &&
                    hasPlaceholders(selectedResult.command.command) &&
                    !query.startsWith(selectedResult.command.alias + ' ')
                ) {
                    setQuery(selectedResult.command.alias + ' ');
                    setSearchMode({
                        mode: 'argument',
                        alias: selectedResult.command.alias,
                        command: selectedResult.command,
                    });
                    return;
                }

                commandToExecute = selectedResult.command;
                const parts = query
                    .trim()
                    .split(/\s+/)
                    .filter((a) => a);
                argsToPass = parts.length <= 1 ? [] : parts;
            }

            const keepWindowOpen = await executeCommand(commandToExecute, argsToPass);
            // null means the run failed: keep what was typed so it can be corrected and retried
            if (keepWindowOpen === null) return;
            resetState();

            if (keepWindowOpen === false) {
                TauriAPI.hideWindow();
            }
        },
        [
            results,
            selectedIndex,
            query,
            executeCommand,
            argumentMode,
            resetState,
            setSearchMode,
            setQuery,
        ],
    );

    return { runSelectedCommand };
};
