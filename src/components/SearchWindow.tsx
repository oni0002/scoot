import React, { useState, useCallback } from 'react';
import { Command } from '../types';
import { TauriAPI } from '../api/tauri';
import { SearchBar } from './SearchBar';
import { SearchResultList } from './SearchResultList';
import { useCommandContext } from '../context/CommandContext';
import { useSearchState } from '../hooks/useSearchState';
import { useKeyboardNavigation } from '../hooks/useKeyboardNavigation';
import { useWindowEvents } from '../hooks/useWindowEvents';

interface SearchWindowProps {
  fuzzyThreshold?: number;
  maxResults?: number;
  onEditCommand?: (command: Command) => void;
  onDeleteCommand?: (command: Command) => void;
  onAddCommand?: () => void;
  onCopyCommand?: (command: Command) => void;
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
  const executeCommand = useCallback((targetIndex?: number) => {
    if (promptMode) {
      // プロンプト部分を除いた引数を抽出
      const promptPrefix = promptMode.prompt + ' ';
      const argsString = query.startsWith(promptPrefix)
        ? query.slice(promptPrefix.length).trim()
        : query.trim();
      const args = argsString ? argsString.split(/\s+/) : [];
      executeContextCommand(promptMode.command, args);
      resetState();

      if (promptMode.command.command === 'scoot://add-command') {
        return;
      }

      TauriAPI.hideWindow();
      return;
    }

    // 引数でインデックスが指定された場合はそれを使用、なければ現在のstateを使用
    const effectiveIndex = targetIndex !== undefined ? targetIndex : selectedIndex;

    if (results.length === 0 || effectiveIndex >= results.length) return;

    const selectedResult = results[effectiveIndex];

    // プロンプトモードへの移行チェック
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

    // 通常の実行
    const parsed = promptProcessor.current.parseInput(query);
    executeContextCommand(selectedResult.command, parsed.args);
    resetState();

    // 内部ビュー切り替えを伴うコマンドの場合はウィンドウを隠さない
    if (selectedResult.command.command === 'scoot://add-command' || selectedResult.command.command === 'scoot://reload') {
      return;
    }

    TauriAPI.hideWindow();
  }, [results, selectedIndex, query, executeContextCommand, promptMode, resetState, setPromptMode, setQuery, setResults, setSelectedIndex, promptProcessor]);

  // Keyboard Navigation
  const { handleKeyDown } = useKeyboardNavigation({
    moveSelection: searchState.moveSelection,
    executeCommand,
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
    executeCommand(index);
  }, [executeCommand, setSelectedIndex]);

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

  const handleOpenCommandsJson = useCallback(async (event: React.MouseEvent) => {
    event.stopPropagation();
    try {
      await TauriAPI.openCommandsJson();
    } catch (error) {
      console.error('Failed to open commands.json:', error);
    }
  }, []);

  const handleOpenConfigJson = useCallback(async (event: React.MouseEvent) => {
    event.stopPropagation();
    try {
      await TauriAPI.openConfigJson();
    } catch (error) {
      console.error('Failed to open config.json:', error);
    }
  }, []);

  const handleShowReadme = useCallback(async (event: React.MouseEvent) => {
    event.stopPropagation();
    try {
      await TauriAPI.openReadme();
    } catch (error) {
      console.error('Failed to open README:', error);
    }
  }, []);

  const handleShowAbout = useCallback(async (event: React.MouseEvent) => {
    event.stopPropagation();
    try {
      const version = await TauriAPI.getVersion();
      await TauriAPI.showMessage(
        `Scoot - Command Launcher\nVersion ${version}\n\nA fast and efficient command launcher for your desktop.`,
        'About Scoot'
      );
    } catch (error) {
      console.error('Failed to show About dialog:', error);
      // Fallback
      alert('Scoot - Command Launcher\n(Version info unavailable)');
    }
  }, []);

  const handleOpenSettings = useCallback((event: React.MouseEvent) => {
    event.stopPropagation();
    alert('Settings dialog not yet implemented.');
  }, []);

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
        onOpenSettings={handleOpenSettings}
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
      />
    </div>
  );
};