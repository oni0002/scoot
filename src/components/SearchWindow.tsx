import React, { useState, useCallback } from 'react';
import { Command } from '../types';
import { TauriAPI } from '../api/tauri';
import { SearchBar } from './SearchBar';
import { SearchResultList } from './SearchResultList';
import { useCommandContext } from '../context/CommandContext';
import { useSearchState } from '../hooks/useSearchState';
import { useKeyboardNavigation } from '../hooks/useKeyboardNavigation';
import { useWindowEvents } from '../hooks/useWindowEvents';

function menuHandler(action: () => Promise<void>, label: string) {
  return async (event: React.MouseEvent) => {
    event.stopPropagation();
    try { await action(); }
    catch (e) { console.error(`Failed to ${label}:`, e); }
  };
}

const handleOpenCommandsJson = menuHandler(() => TauriAPI.openCommandsJson(), 'open commands.json');
const handleOpenConfigJson = menuHandler(() => TauriAPI.openConfigJson(), 'open config.json');
const handleShowReadme = menuHandler(() => TauriAPI.openReadme(), 'open README');
const handleShowAbout = async (event: React.MouseEvent) => {
  event.stopPropagation();
  try {
    const version = await TauriAPI.getVersion();
    await TauriAPI.showMessage(
      `Scoot - Command Launcher\nVersion ${version}\n\nA fast and efficient command launcher for your desktop.`,
      'About Scoot'
    );
  } catch (error) {
    console.error('Failed to show About dialog:', error);
    alert('Scoot - Command Launcher\n(Version info unavailable)');
  }
};

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
  const { commands, executeCommand: executeContextCommand } = useCommandContext();

  // Search Logic & State
  const searchState = useSearchState(commands, fuzzyThreshold, maxResults);
  const {
    query, setQuery, results, setResults, selectedIndex, setSelectedIndex,
    promptMode, setPromptMode, resetState, inputRef, promptProcessor
  } = searchState;

  // UI State for mouse hover
  const [hoveredIndex, setHoveredIndex] = useState<number | null>(null);

  // Window Events (Focus, Shortcuts, Click Outside)
  useWindowEvents(isDialogOpen, resetState, inputRef);

  // Command Execution Logic
  const runSelectedCommand = useCallback((targetIndex?: number) => {
    let commandToExecute: Command;
    let argsToPass: string[] = [];

    if (promptMode) {
      commandToExecute = promptMode.command;
      argsToPass = promptProcessor.current.parseInput(query).args;
    } else {
      const effectiveIndex = targetIndex !== undefined ? targetIndex : selectedIndex;
      if (results.length === 0 || effectiveIndex >= results.length) return;

      const selectedResult = results[effectiveIndex];

      if (selectedResult.command.prompt && !query.startsWith(selectedResult.command.prompt + ' ')) {
        setPromptMode({
          prompt: selectedResult.command.prompt,
          command: selectedResult.command
        });
        setQuery(selectedResult.command.prompt + ' ');
        setResults([]);
        setSelectedIndex(0);
        return;
      }

      commandToExecute = selectedResult.command;
      argsToPass = promptProcessor.current.parseInput(query).args;
    }

    executeContextCommand(commandToExecute, argsToPass);
    resetState();

    if (commandToExecute.command === 'scoot://add-command' || commandToExecute.command === 'scoot://reload') {
      return;
    }

    TauriAPI.hideWindow();
  }, [results, selectedIndex, query, executeContextCommand, promptMode, resetState, setPromptMode, setQuery, setResults, setSelectedIndex, promptProcessor]);

  // Keyboard Navigation
  const { handleKeyDown } = useKeyboardNavigation({
    moveSelection: searchState.moveSelection,
    executeCommand: runSelectedCommand,
    resetState,
    query,
    setQuery,
    setResults,
    setSelectedIndex,
    promptMode
  });

  // Action Handlers
  const handleResultClick = useCallback((index: number, event?: React.MouseEvent) => {
    if (event && (event.target as HTMLElement).closest('.dropdown, .dropdown-content')) {
      return;
    }
    setSelectedIndex(index);
    runSelectedCommand(index);
  }, [runSelectedCommand, setSelectedIndex]);

  const handleAddCommand = useCallback((event: React.MouseEvent) => {
    event.stopPropagation();
    onAddCommand?.();
  }, [onAddCommand]);

  const handleEditCommand = useCallback((command: Command, event: React.MouseEvent) => {
    event.stopPropagation();
    onEditCommand?.(command);
  }, [onEditCommand]);

  const handleDeleteCommand = useCallback((command: Command, event: React.MouseEvent) => {
    event.stopPropagation();
    onDeleteCommand?.(command);
  }, [onDeleteCommand]);

  const handleCopyCommand = useCallback((command: Command, event: React.MouseEvent) => {
    event.stopPropagation();
    onCopyCommand?.(command);
  }, [onCopyCommand]);

  const handleReload = useCallback((event: React.MouseEvent) => {
    event.stopPropagation();
    onReloadCommands?.();
  }, [onReloadCommands]);

  return (
    <div className="h-full flex flex-col p-4">
      <SearchBar
        query={query}
        onQueryChange={searchState.handleQueryChange}
        onKeyDown={handleKeyDown}
        inputRef={inputRef}
        promptMode={promptMode}
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
        promptMode={promptMode}
        promptArgs={promptMode ? (
          query.startsWith(promptMode.prompt + ' ')
            ? query.slice(promptMode.prompt.length + 1).trim().split(/\s+/).filter(a => a)
            : []
        ) : []}
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