import React, { useState, useCallback, useMemo, useRef } from 'react';
import { Command } from '../types';
import { TauriAPI } from '../tauri';
import { NOTIFICATION_DURATION, resultOptionId } from '../constants';
import { RowAction, rowActionsFor } from '../rowActions';
import { getErrorMessage } from '../error';
import { hasPlaceholders, parseArgs, substituteArgs } from '../command';
import { SearchBar } from './SearchBar';
import { SearchResultList } from './SearchResultList';
import { SearchHints } from './SearchHints';
import { useCommandContext } from '../context/CommandContext';
import { useNotificationContext } from '../context/NotificationContext';
import { useSearchState } from '../hooks/useSearchState';
import { useCommandExecution } from '../hooks/useCommandExecution';
import { useKeyboardNavigation } from '../hooks/useKeyboardNavigation';
import { useWindowEvents } from '../hooks/useWindowEvents';
import { useRestoreFocus } from '../hooks/useRestoreFocus';

interface SearchWindowProps {
    fuzzyThreshold?: number;
    maxResults?: number;
    usageHistory?: boolean;
    frequentFirst?: boolean;
    onEditCommand?: (command: Command) => void;
    onDuplicateCommand?: (command: Command) => void;
    onDeleteCommand?: (command: Command) => void;
    onAddCommand?: () => void;
    onCopyCommand?: (command: Command) => void;
    onAliasCommand?: (command: Command) => void;
    onIgnoreCommand?: (command: Command) => void;
    onReloadCommands?: () => void;
    onOpenConfig?: () => void;
    isDialogOpen?: boolean;
}

export const SearchWindow: React.FC<SearchWindowProps> = ({
    fuzzyThreshold = 0.5,
    maxResults = 10,
    usageHistory = true,
    frequentFirst = true,
    onEditCommand,
    onDuplicateCommand,
    onDeleteCommand,
    onAddCommand,
    onCopyCommand,
    onAliasCommand,
    onIgnoreCommand,
    onReloadCommands,
    onOpenConfig,
    isDialogOpen = false,
}) => {
    const { commands, usage, executeCommand: rawExecuteCommand } = useCommandContext();
    const { showError } = useNotificationContext();

    const executeContextCommand = useCallback(
        async (command: Command, args: string[]) => {
            try {
                return await rawExecuteCommand(command, args);
            } catch (err) {
                const message = getErrorMessage(err);
                console.error(`Failed to execute command "${command.name}":`, message);
                showError(
                    `Could not run "${command.name}": ${message}`,
                    NOTIFICATION_DURATION.LONG,
                    command.source === 'user'
                        ? { label: 'Edit', onClick: () => onEditCommand?.(command) }
                        : undefined,
                );
                return null;
            }
        },
        [rawExecuteCommand, showError, onEditCommand],
    );

    const searchState = useSearchState(
        commands,
        fuzzyThreshold,
        maxResults,
        usageHistory ? usage : undefined,
        usageHistory && frequentFirst,
    );
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
    const clearHover = useCallback(() => setHoveredIndex(null), []);
    const focusSearch = useCallback(() => inputRef.current?.focus(), [inputRef]);
    const containerRef = useRef<HTMLDivElement>(null);

    useWindowEvents(isDialogOpen, resetState, inputRef, containerRef);

    // A row menu that closes because the pointer left takes its focus with it; give it back so the
    // next keystroke still lands in the search field.
    useRestoreFocus(inputRef, hoveredIndex === null && !isDialogOpen);

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

    const aliasExample = useMemo(
        () => commands.find((c) => c.alias && hasPlaceholders(c.command))?.alias,
        [commands],
    );

    const args = useMemo(
        () => (argumentMode ? parseArgs(query, argumentMode.alias) : []),
        [argumentMode, query],
    );

    // In argument mode the thing worth copying is the command as it will run, not its template
    const copyExpanded = useCallback(() => {
        if (!argumentMode) return;
        const { command } = argumentMode;
        onCopyCommand?.({
            ...command,
            command: substituteArgs(command.command, args, command.category === 'url'),
        });
    }, [argumentMode, args, onCopyCommand]);

    const handleCopySelectedCommand = useCallback(() => {
        if (argumentMode) {
            copyExpanded();
            return;
        }
        if (selectedIndex < 0 || selectedIndex >= results.length) return;
        const command = results[selectedIndex].command;
        if (command) onCopyCommand?.(command);
    }, [argumentMode, copyExpanded, selectedIndex, results, onCopyCommand]);

    // The ignored row is gone afterwards, so go back to a clean search instead of an empty result
    const handleIgnore = useCallback(
        (command: Command) => {
            onIgnoreCommand?.(command);
            resetState();
        },
        [onIgnoreCommand, resetState],
    );

    // One path for every row action, whether picked from the row or by its shortcut
    const runRowAction = useCallback(
        (action: RowAction, command: Command) => {
            if (!rowActionsFor(command).includes(action)) return;
            if (action === 'copy') onCopyCommand?.(command);
            else if (action === 'edit') onEditCommand?.(command);
            else if (action === 'duplicate') onDuplicateCommand?.(command);
            else if (action === 'delete') onDeleteCommand?.(command);
            else if (action === 'alias') onAliasCommand?.(command);
            else handleIgnore(command);
        },
        [
            onCopyCommand,
            onEditCommand,
            onDuplicateCommand,
            onDeleteCommand,
            onAliasCommand,
            handleIgnore,
        ],
    );

    const handleRowAction = useCallback(
        (action: Exclude<RowAction, 'copy'>) => {
            const command = results[selectedIndex]?.command;
            if (command) runRowAction(action, command);
        },
        [results, selectedIndex, runRowAction],
    );

    const handleRowClickAction = useCallback(
        (action: RowAction, command: Command, event: React.MouseEvent) => {
            event.stopPropagation();
            runRowAction(action, command);
            if (action === 'copy') focusSearch();
        },
        [runRowAction, focusSearch],
    );

    const { handleKeyDown } = useKeyboardNavigation({
        moveSelection: searchState.moveSelection,
        executeCommand: runSelectedCommand,
        copySelectedCommand: handleCopySelectedCommand,
        onRowAction: handleRowAction,
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

    // Menu actions run from the ⋮ menu; a failure must reach the user, not just the console.
    const menuHandler =
        (action: () => Promise<void>, label: string) => async (event: React.MouseEvent) => {
            event.stopPropagation();
            try {
                await action();
            } catch (e) {
                console.error(`Failed to ${label}:`, e);
                showError(`Could not ${label}: ${getErrorMessage(e)}`);
            }
        };
    const handleOpenCommandsJson = menuHandler(
        () => TauriAPI.openCommandsJson(),
        'open commands.json',
    );
    const handleOpenConfigJson = menuHandler(() => TauriAPI.openConfigJson(), 'open config.json');
    const handleShowAbout = menuHandler(async () => {
        const version = await TauriAPI.getVersion();
        await TauriAPI.showMessage(
            `Scoot - Command Launcher
Version ${version}

A fast and efficient command launcher for your desktop.`,
            'About Scoot',
        );
    }, 'show the About dialog');

    const handleReload = useCallback(
        (event: React.MouseEvent) => {
            event.stopPropagation();
            onReloadCommands?.();
        },
        [onReloadCommands],
    );

    return (
        <div ref={containerRef} className="h-full flex flex-col p-4">
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
                onShowAbout={handleShowAbout}
                onOpenConfig={onOpenConfig}
                hasResults={results.length > 0}
                activeDescendantId={results.length > 0 ? resultOptionId(selectedIndex) : undefined}
            />

            {/* Announces the result count to screen readers as the query changes */}
            <div className="sr-only" role="status" aria-live="polite">
                {searchMode.mode === 'search' && query.trim()
                    ? `${results.length} ${results.length === 1 ? 'result' : 'results'}`
                    : ''}
            </div>

            <SearchResultList
                results={results}
                selectedIndex={selectedIndex}
                hoveredIndex={hoveredIndex}
                argumentMode={argumentMode}
                args={args}
                query={query}
                onResultClick={handleResultClick}
                onMouseEnter={setHoveredIndex}
                onMouseLeave={clearHover}
                onAction={handleRowClickAction}
                aliasExample={aliasExample}
                onCopyExpanded={copyExpanded}
                focusSearch={focusSearch}
            />

            <SearchHints
                mode={searchMode.mode === 'idle' && results.length > 0 ? 'search' : searchMode.mode}
                command={
                    searchMode.mode !== 'argument' ? results[selectedIndex]?.command : undefined
                }
            />
        </div>
    );
};
