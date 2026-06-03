import React, { useState, useCallback } from 'react';
import { Command } from '../types';
import { TauriAPI } from '../tauri';
import { NOTIFICATION_DURATION } from '../constants';
import { SearchBar } from './SearchBar';
import { SearchResultList } from './SearchResultList';
import { useCommandContext } from '../context/CommandContext';
import { useNotificationContext } from '../context/NotificationContext';
import { useSearchState } from '../hooks/useSearchState';
import { useCommandExecution } from '../hooks/useCommandExecution';
import { useKeyboardNavigation } from '../hooks/useKeyboardNavigation';
import { useWindowEvents } from '../hooks/useWindowEvents';

function menuHandler(action: () => Promise<void>, label: string) {
    return async (event: React.MouseEvent) => {
        event.stopPropagation();
        try {
            await action();
        } catch (e) {
            console.error(`Failed to ${label}:`, e);
        }
    };
}

const handleOpenCommandsJson = menuHandler(() => TauriAPI.openCommandsJson(), 'open commands.json');
const handleOpenConfigJson = menuHandler(() => TauriAPI.openConfigJson(), 'open config.json');
const handleShowReadme = menuHandler(() => TauriAPI.openReadme(), 'open README');
const handleShowAbout = menuHandler(async () => {
    const version = await TauriAPI.getVersion();
    await TauriAPI.showMessage(
        `Scoot - Command Launcher\nVersion ${version}\n\nA fast and efficient command launcher for your desktop.`,
        'About Scoot',
    );
}, 'show About dialog');

interface SearchWindowProps {
    fuzzyThreshold?: number;
    maxResults?: number;
    onEditCommand?: (command: Command) => void;
    onDeleteCommand?: (command: Command) => void;
    onAddCommand?: () => void;
    onCopyCommand?: (command: Command) => void;
    onIgnoreCommand?: (command: Command) => void;
    onReloadCommands?: () => void;
    isDialogOpen?: boolean;
}

export const SearchWindow: React.FC<SearchWindowProps> = ({
    fuzzyThreshold = 0.5,
    maxResults = 10,
    onEditCommand,
    onDeleteCommand,
    onAddCommand,
    onCopyCommand,
    onIgnoreCommand,
    onReloadCommands,
    isDialogOpen = false,
}) => {
    const { commands, executeCommand: rawExecuteCommand } = useCommandContext();
    const { showError } = useNotificationContext();

    const executeContextCommand = useCallback(
        async (command: Command, args: string[]) => {
            const result = await rawExecuteCommand(command, args);
            if (result === null) {
                showError(`Failed to execute "${command.name}".`, NOTIFICATION_DURATION.LONG);
            }
            return result;
        },
        [rawExecuteCommand, showError],
    );

    const searchState = useSearchState(commands, fuzzyThreshold, maxResults);
    const {
        query,
        setQuery,
        searchMode,
        setSearchMode,
        results,
        selectedIndex,
        argumentMode,
        resetState,
        inputRef,
    } = searchState;

    const [hoveredIndex, setHoveredIndex] = useState<number | null>(null);

    useWindowEvents(isDialogOpen, resetState, inputRef);

    const { runSelectedCommand } = useCommandExecution({
        query,
        results,
        selectedIndex,
        argumentMode,
        setQuery,
        setSearchMode,
        resetState,
        executeCommand: executeContextCommand,
    });

    const handleCopySelectedCommand = useCallback(() => {
        if (selectedIndex < 0 || selectedIndex >= results.length) return;
        const command = results[selectedIndex].command;
        if (command) onCopyCommand?.(command);
    }, [selectedIndex, results, onCopyCommand]);

    const { handleKeyDown } = useKeyboardNavigation({
        moveSelection: searchState.moveSelection,
        executeCommand: runSelectedCommand,
        copySelectedCommand: handleCopySelectedCommand,
        resetState,
        query,
        setQuery,
        searchMode,
        setSearchMode,
        inputRef,
    });

    const handleResultClick = useCallback(
        (index: number, event?: React.MouseEvent) => {
            if (event && (event.target as HTMLElement).closest('.dropdown, .dropdown-content')) {
                return;
            }
            setSearchMode((prev) =>
                prev.mode === 'search' ? { ...prev, selectedIndex: index } : prev,
            );
            runSelectedCommand(index);
        },
        [runSelectedCommand, setSearchMode],
    );

    const handleAddCommand = useCallback(
        (event: React.MouseEvent) => {
            event.stopPropagation();
            onAddCommand?.();
        },
        [onAddCommand],
    );

    const handleEditCommand = useCallback(
        (command: Command, event: React.MouseEvent) => {
            event.stopPropagation();
            onEditCommand?.(command);
        },
        [onEditCommand],
    );

    const handleDeleteCommand = useCallback(
        (command: Command, event: React.MouseEvent) => {
            event.stopPropagation();
            onDeleteCommand?.(command);
        },
        [onDeleteCommand],
    );

    const handleCopyCommand = useCallback(
        (command: Command, event: React.MouseEvent) => {
            event.stopPropagation();
            onCopyCommand?.(command);
        },
        [onCopyCommand],
    );

    const handleReload = useCallback(
        (event: React.MouseEvent) => {
            event.stopPropagation();
            onReloadCommands?.();
        },
        [onReloadCommands],
    );

    return (
        <div className="h-full flex flex-col p-4">
            <SearchBar
                query={query}
                onQueryChange={searchState.handleQueryChange}
                onKeyDown={handleKeyDown}
                inputRef={inputRef}
                argumentMode={argumentMode}
                onAddCommand={handleAddCommand}
                onReloadCommands={handleReload}
                onOpenCommandsJson={handleOpenCommandsJson}
                onOpenConfigJson={handleOpenConfigJson}
                onShowReadme={handleShowReadme}
                onShowAbout={handleShowAbout}
            />

            <SearchResultList
                results={results}
                selectedIndex={selectedIndex}
                hoveredIndex={hoveredIndex}
                argumentMode={argumentMode}
                args={
                    argumentMode
                        ? query.startsWith(argumentMode.alias + ' ')
                            ? query
                                  .slice(argumentMode.alias.length + 1)
                                  .trim()
                                  .split(/\s+/)
                                  .filter((a) => a)
                            : []
                        : []
                }
                query={query}
                onResultClick={handleResultClick}
                onMouseEnter={setHoveredIndex}
                onMouseLeave={() => setHoveredIndex(null)}
                onCopy={handleCopyCommand}
                onEdit={handleEditCommand}
                onDelete={handleDeleteCommand}
                onIgnore={onIgnoreCommand}
            />
        </div>
    );
};
